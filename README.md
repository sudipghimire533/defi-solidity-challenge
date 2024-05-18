# DeFi-Solidity Challenge

# Overall requirements
- a erc-20 smart contract for FUD token with limited supply of 1.5 million
- another erc-20 token named WIN with unlimited supply
- another contract named Airplay. This will have deposit/withdraw function and events
- a backend service that keeps track of deposits/withdrawal in Airplay and every x block do airdrop.
Airdrop value being caluclate as 5% of stored power and power being amount*life

# Tools Used
- Solidity for writing smart contract ( As per requirement )

- Javascript for contract testing ( Have large ecosystem for smart contarct interaction mainly truffle for testing )
- Hardhat for managing solidity test ( Mainstream solidity interface since truffle is discounted )
- Rust ( Goto choice for cryptographic related service. Have strong type system for accuracy and various option for secure private key storage)

# Smart Contract Layout
All of the contracts are written in single solidity file for simplicity purpose. Firstly,
The ERC-20 standard is written seperately as a abstraction for FUD and WIN to inherit from.

FUD contract simply inherit from ERC-20 and provides no additional functionality

WIN contract inherits ERC-20 and also add external mint function to allow minting. Minting is only allowed for configured minter.
This is enforced by OnlyMinter modifier

AirPlay Contract have a deposit and withdraw function to deposit fund and withdraw fund. These will directly
credit or debit from FUD balance. Before depositing, user have to set allowance for AirplayContract to lock that amount.
This part is best handled on front end and is not in scope of thi assignment.

Every Deposit and Withdraw call also emit's it's respective event for backend to track. This include amount, address and block_num
which are all essential to calculate airdrop amount. This will be discussed later.


# Truffle testing
We have covered basic testing for all expected behaviour. Correct implementation satisfies:
- all erc-20 functionality is being enfored in FUD and WIN including proper event emision
- minting of FUD should not be possible and should be capped at 1.5 million
- mintion of WIN should be possible only by the configured minter address
- Airplay deposit and withdraw function if succeed should change the address FUD balance directly
- If use have not set enough allowance, deposit should fail
- If user tries to withdraw more fund than deposited, call should fail
- User should be able to deposit any number times with any value
- Use should be able to withdraw any amount any nuber of time as long as they have enough deposit
- Amount deposited and withdrawn in same airdrop interval should also reflect the reward amount


# Airdrop Backend
## Logic flow:
- connect to Airplay contract and listen for all Withdrawn/ Deposited events
- For each events add or remove the amount from user total amount based on weather it is deposited or withdrawen
- When storing award amount also take care of the block which was it's executed on:
For example:
- New airdrop interval starts as 100 and ends at 200. Alice deposited 50FUD at 140 and withdrawn at 180. His 50FUD stayed for 40 blocks. so amount should be 50*40 power. and we only reward 5% of power so final reward amount is 0.05*50*40.
- In same case, if bob deposited 10FUD at block 140 and never withdraw, his award will be 0.05*( 10 *(200-140)) for first airdrop then 0.05*10*100 for every other interval ( 100 being block interval )
- once a interval is finished calculate the reward amount and mint respective amount of WIN to user address. This task is a seperate tokio thread hence wont block the main subscribe event thread. And we have to call the mint function from configured minter address in Airplay Contract.

# First Step to improve
- split each contract into it's own solidity file. This make it's easier in terms of maintainance
- test more verbosely all the contrac behaviour and make each test in it's seperate chai scope. This makes it easier to manage and monitor test coverage
- Optimise contract for potentional gas saving and speed of execution. Example:
 1) unchecked state mutation if we are already checking the pre condition
 2) shared functionality can be introduced and make a shared interface to make solidity bytecode smaller in size: Example: introduce _spend_balance() function instead of manually increasing/decreasing sender/receiver address. Define this once and resue.
 3) In Airplay contract, balance and block number can be stored in intiger storage splitting into lower bits and upper bits instead of two seoerate storage. This will save storage cost. However be careful, we have to be considerate if these two can be fit and is packing/unpacking worthwhile. 
 4) Have seperate Minted event instead of using Transfer event
 5) And many more to be discussed..
- Split rust backend service into more organised project layout. Example: having event_subscriber.rs, minter.rs as seperate module.
- Have a config file to pass Contract address before each run
- Have a strategy to store minter address, this can be one of many:
1) store in encrypted file and read every time in a secure way. Read every time it's needed and do not store in memory
2) Store in external provider like AWS KMS or Trezor or YubiHSM2
- instead of storing all the rewards in memory, set them in database so that this is presistant
- error handle and report if call are being sucessfully made or not
- Replace all .unwrap()/epect() call with error handeling
- Instead of manually pasting contract ABI implement to read the ABI json from hardhat
- And many more..

# Disclamer
Not all these test are covered. Basics are and other edge case can be discussed as well. Smart contract is not modified for any purpose including nor storage cost, nor execution speed nor gas cost. Backend code structule can be more extinsible splitting program into modules and more carefully managing threads and keys.



This project when I did it, my only goal was to demonestrate the skill to understand the requirement and have a working prototype quickly. Technoloigy used and strategy used can be of course varied dependnig on the requirement. However someone who goes through the assignmentcan take this as barely a few hour work and main intention is to establish a context to talk in our upcoming interview.  Hope to see you soon :)
