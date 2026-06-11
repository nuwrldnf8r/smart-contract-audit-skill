// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IERC20 {
    function transfer(address,uint256) external returns (bool);
    function transferFrom(address,address,uint256) external returns (bool);
    function balanceOf(address) external view returns (uint256);
}

/// @notice Minimal tokenized yield vault (ERC4626-like). Shares represent a claim on assets.
contract YieldVault {
    IERC20 public immutable asset;
    uint256 public totalShares;
    mapping(address => uint256) public shares;

    constructor(IERC20 _asset) { asset = _asset; }

    function totalAssets() public view returns (uint256) {
        return asset.balanceOf(address(this));
    }

    // Subtle H1: no virtual shares / no minimum initial liquidity. The first depositor
    // can mint 1 wei of shares, donate a large amount directly to the vault to inflate
    // totalAssets, and steal a later depositor's funds via rounding.
    function deposit(uint256 assets) external returns (uint256 sh) {
        uint256 ta = totalAssets();
        sh = totalShares == 0 ? assets : (assets * totalShares) / ta;
        asset.transferFrom(msg.sender, address(this), assets);
        shares[msg.sender] += sh;
        totalShares += sh;
    }

    // Subtle H2: rounding direction favors the withdrawer. Shares burned are floored,
    // so a user repeatedly withdrawing small amounts burns fewer shares than the assets
    // they remove are worth, slowly draining other holders.
    function withdraw(uint256 assets) external returns (uint256 sh) {
        sh = (assets * totalShares) / totalAssets();   // floor
        shares[msg.sender] -= sh;
        totalShares -= sh;
        asset.transfer(msg.sender, assets);
    }

    // Subtle H3: redeem performs the external asset transfer BEFORE updating shares.
    // If `asset` has a transfer callback (ERC777/ERC1155-style), the recipient can
    // re-enter redeem/withdraw while their share balance is still intact.
    function redeem(uint256 sh) external returns (uint256 assets) {
        assets = (sh * totalAssets()) / totalShares;
        asset.transfer(msg.sender, assets);
        shares[msg.sender] -= sh;
        totalShares -= sh;
    }

    // Subtle H4: this view is the canonical price other protocols read for collateral
    // valuation. It has no reentrancy protection and reflects the live token balance,
    // so during a redeem callback (H3) it returns a manipulated price (read-only reentrancy).
    function pricePerShare() external view returns (uint256) {
        return totalShares == 0 ? 1e18 : (totalAssets() * 1e18) / totalShares;
    }
}
