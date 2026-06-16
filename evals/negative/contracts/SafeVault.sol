// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @notice Negative-eval fixture: contains patterns that LOOK like classic bugs but are SAFE.
///         A well-calibrated audit should NOT report these as vulnerabilities.
///         (Not a reference implementation; minimized for the eval.)
interface IPair {
    function getReserves() external view returns (uint112 r0, uint112 r1, uint32 ts);
}

contract SafeVault is Ownable2Step {
    using SafeERC20 for IERC20;

    IERC20 public immutable asset;
    IPair public immutable displayPair; // spot source used for DISPLAY ONLY, never accounting
    mapping(address => uint256) public balanceOf;
    uint256 public feeBps;              // bounded admin parameter
    uint256 public constant MAX_FEE_BPS = 100; // hard cap: 1%

    constructor(IERC20 _asset, IPair _displayPair) Ownable(msg.sender) {
        asset = _asset;
        displayPair = _displayPair;
    }

    function deposit(uint256 amount) external {
        // balance credited from the measured delta, so fee-on-transfer tokens can't over-credit
        uint256 before = asset.balanceOf(address(this));
        asset.safeTransferFrom(msg.sender, address(this), amount);
        uint256 received = asset.balanceOf(address(this)) - before;
        balanceOf[msg.sender] += received;
    }

    function withdraw(uint256 amount) external {
        // checks-effects-interactions: state is updated BEFORE the external transfer,
        // so the external call cannot be used to re-enter and double-withdraw.
        require(balanceOf[msg.sender] >= amount, "insufficient");
        balanceOf[msg.sender] -= amount;
        asset.safeTransfer(msg.sender, amount);
    }

    /// @dev Admin power is bounded: cannot exceed MAX_FEE_BPS, and Ownable2Step prevents
    ///      transferring ownership to a wrong/zero address. Not a centralization risk of note.
    function setFeeBps(uint256 bps) external onlyOwner {
        require(bps <= MAX_FEE_BPS, "fee too high");
        feeBps = bps;
    }

    /// @notice Spot price from a single pair — but it is ONLY returned for off-chain display.
    ///         It never feeds collateral, borrow, or liquidation math, so its manipulability
    ///         carries no on-chain exploit. Flagging it as oracle manipulation is a false positive.
    function displayPriceX18() external view returns (uint256) {
        (uint112 r0, uint112 r1,) = displayPair.getReserves();
        require(r0 > 0, "no liquidity");
        return (uint256(r1) * 1e18) / uint256(r0);
    }
}
