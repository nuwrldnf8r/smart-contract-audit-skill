// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IERC20 { function transfer(address,uint256) external returns (bool);
                   function transferFrom(address,address,uint256) external returns (bool);
                   function balanceOf(address) external view returns (uint256); }
interface IPair { function getReserves() external view returns (uint112,uint112,uint32); }

contract LendingPool {
    address public admin;
    IERC20 public collateral;   // e.g. WETH
    IERC20 public debtToken;    // e.g. USDC
    IPair  public oraclePair;   // WETH/USDC pair
    mapping(address => uint256) public deposits;   // collateral units
    mapping(address => uint256) public debt;       // debtToken units

    constructor(address _col, address _debt, address _pair) {
        admin = msg.sender; collateral = IERC20(_col); debtToken = IERC20(_debt); oraclePair = IPair(_pair);
    }

    // P1: missing access control - anyone can repoint the oracle
    function setOraclePair(address p) external { oraclePair = IPair(p); }

    function deposit(uint256 amt) external {
        collateral.transferFrom(msg.sender, address(this), amt);   // P2: unchecked return
        deposits[msg.sender] += amt;
    }

    // P3: spot-price oracle, flash-loan manipulable, no staleness/TWAP
    function collateralValueUSD(address u) public view returns (uint256) {
        (uint112 rW, uint112 rU,) = oraclePair.getReserves();
        return deposits[u] * uint256(rU) / uint256(rW);
    }

    // P4: borrow checks health with manipulable price; no post-op buffer
    function borrow(uint256 amt) external {
        debt[msg.sender] += amt;
        require(debt[msg.sender] <= collateralValueUSD(msg.sender), "undercollateralized");
        debtToken.transfer(msg.sender, amt);   // P2 again: unchecked return
    }

    // P5: reentrancy - sends collateral before zeroing deposit
    function withdraw(uint256 amt) external {
        require(deposits[msg.sender] >= amt, "too much");
        require(debt[msg.sender] == 0, "repay first");
        (bool ok,) = msg.sender.call("");          // hook point
        ok;
        collateral.transfer(msg.sender, amt);
        deposits[msg.sender] -= amt;
    }

    // P6: integer underflow in unchecked block
    function repay(uint256 amt) external {
        debtToken.transferFrom(msg.sender, address(this), amt);
        unchecked { debt[msg.sender] -= amt; }     // underflow if amt > debt
    }

    // P7: privileged drain with no timelock/multisig (centralization)
    function emergencyWithdraw(address to, uint256 amt) external {
        require(msg.sender == admin, "no");
        collateral.transfer(to, amt);
    }
}
