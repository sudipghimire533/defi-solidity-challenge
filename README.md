# DeFi-Solidity Challenge

## Introduction

The goal of this challenge is to gauge skills in Solidity smart development and experience in interacting with Solidity smart contracts. The idea is to create a simple set of smart contracts and a small backend service to interact with them.

Important points:

* **Document your decisions**: your solution should include a README that justifies the choice of any frameworks, libraries, or design decisions you used to solve the problem
* **Unit tests**: testing in both Solidity and the backend is an essential part of any solution
* **Keep it simple**: you don't need to design a production system, so don't worry about setting up a database or complicated messaging systems. Do, however, create a solution that would later be extendible with any features you forsee being important for future production development.
* **Try to minimize gas fees**: first try to get a working solution, and if you have extra time, consider playing around with your contract to get the minimal gas fees necessary on deposits and withdraws.
* **Discuss private key management**: you don't need to implement any advanced private key handling in your solution, however, do discuss how you would go about securing a private key in your solution, and make sure the code is extensible for such a scenario.

## The Challenge

We are creating a system of two tokens:

* **FUD** token: An ERC20 token with a max supply of 1.5 M token.
* **WIN** token: A mintable ERC20 reward token with no max supply.

We want to provide some utility for the FUD token by allowing user's to deposit their FUD tokens in a EVM smart contract and earn rewards. The rewards for depositing FUD tokens will be distributed in the form of newly minted WIN tokens. What can user's do with their WIN tokens? Well, nothing. They have clearly already won.

Win tokens will be distributed by by airdrop every X blocks to each user who has had FUD tokens deposited during this interval. A separate backend service will airdrop WIN tokens by minting them directly to the depositor's address. The amount of airdropped tokens is equal to 5% of the average FUD token deposit over the last X blocks (X should be configurable).

```
# WIN tokens in airdrop = 0.05 * (# FUD tokens deposited) * (# blocks deposited) / ( total # blocks)
```

As an example, if the airdrop interval is 100 blocks, and a user deposited 10 total FUD tokens for 40 blocks, and 20 total FUD tokens for the remailing 60 blocks, then the WIN token airdrop would be:
```
((10 FUD)*(40 blocks) + (20 FUD)*(60 blocks))*0.05/(100 blocks) = 0.8 WIN
```

Every deposit or withdraw of a FUD token should emit an event that can be picked up by our backend. The event should contain the number of FUD tokens depositied or withdrawn, and the address of the depositer (you can also include any other information you'd like in the event, such as the new deposit balance).

The backend should monitor the on-chain contracts for any events, and airdrop the WIN tokens to the depositer's address every X blocks.

Overall, there should be three Solidity contracts with the following interfaces. The token contracts should both implement the standard ERC20 interface defined [here](https://ethereum.org/en/developers/docs/standards/tokens/erc-20/).

```
pragma solidity ^0.8.13

interface FudToken {
	// ... standard ERC20 interface
}

interface WinToken {
	// ... standard ERC20 interface
	
	// the WIN token is also mintable, so we include the following with the onlyMinter modifier
	function mint(address account, uint256 amount) public returns(bool);
}

interface AirVault {
	// lock tokens in the AirVault contract
	deposit(uint256 amount) public returns(bool);

	// withdraw deposited tokens
	withdraw(uint256 amount) public returns(bool);
	
	// provides how many tokens a specific address has deposited
	lockedBalanceOf(address account) external view returns(uint256);
}
```

The backend should have control of a private key corresponding to the minter role on the WIN token. Only the controller of this private key should be able to mint WIN tokens. This will allow it to airdrop tokens to the deposit addresses after the block interval.

The backend can be written in any language of your choice, though you should provide a short justification for why you chose that language.

Your solution should include an implementation of the three above smart contracts, the backend, and a README that ties it all together. For simplicity you can develop using a local Ganache instance, or any EVM testnet (ie. Avalanche C-Chain, Polygon, Goerli etc).