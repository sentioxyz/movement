// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;
import "forge-std/console.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { IERC20 } from "@openzeppelin/contracts/interfaces/IERC20.sol";
import { EnumerableSet } from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract MovementStakingStorage {

    using SafeERC20 for IERC20;
    using EnumerableSet for EnumerableSet.AddressSet;

    // the token used for staking
    IERC20 public token;

    mapping(address domain => uint256 epochDuration) public epochDurationByDomain;
    mapping(address domain => uint256 currentEpoch) public currentEpochByDomain;
    mapping(address domain => EnumerableSet.AddressSet attester) internal attestersByDomain;
    mapping(address domain => EnumerableSet.AddressSet custodian) internal custodiansByDomain;

    // preserved records of stake by address per epoch
    mapping(address domain => 
        mapping(uint256 => 
            mapping(address => 
                mapping(address => uint256)))) public epochStakesByDomain;

    // preserved records of unstake by address per epoch
    mapping(address domain => 
        mapping(uint256 => 
            mapping(address =>
                mapping(address => uint256))))  public epochUnstakesByDomain;

    // track the total stake of the epoch (computed at rollover)
    mapping(address domain =>
        mapping(uint256 =>
            mapping(address=> uint256))) public epochTotalStakeByDomain;

}