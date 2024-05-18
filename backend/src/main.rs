use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use ethers::prelude::*;

// This is the default address
// contract is deployed by this address
const MINTER: (&str, &str) = (
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
);

abigen!(
    WinToken,
    r#"[
        function name() public view returns (string memory)
        function symbol() public view returns (string memory)
        function decimals() public pure returns (uint8)
        function totalSupply() public view returns (uint256)
        function balanceOf(address account) public view returns (uint256)
        function allowance(address owner, address spender ) public view returns (uint256)
        function _mint(address account, uint256 amount) internal virtual
        function transfer(address to, uint256 amount) external returns (bool success)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom(address from, address to, uint256 amount) external returns (bool)

        event Transfer(address indexed from, address indexed to, uint256 value)
        event Approval( address indexed owner, address indexed spender, uint256 value )

        function mint(address to, uint256 amount) external OnlyMinter returns (bool)
    ]"#,
);

abigen!(
    AirVault,
    r#"[
        function deposit(uint256 amount) external returns (bool)
        function withdraw(uint256 amount) public returns (bool)
        function lockedBalanceOf(address account) external view returns (uint256)

        event Deposited(address indexed account, uint indexed block_num, uint256 amount)
        event Withdrawn(address indexed account, uint indexed block_num, uint256 amount)
    ]"#;
);

#[tokio::main]
async fn main() {
    // For these addresst to be valid:
    // - start fresh new hardhat node
    // - deploy FudToken Contract
    // - deploy WinToken Contract
    // - deploy AirVault Contract
    // in same order without any pre/mid/ transaction in between
    let minter = H160::from_str(MINTER.0).unwrap();
    let _fud_token_addr = ethers::core::utils::get_contract_address(minter, 0);
    let win_token_addr = ethers::core::utils::get_contract_address(minter, 1);
    let airvault_addr = ethers::core::utils::get_contract_address(minter, 2);

    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = Arc::new(provider);

    let win_token = WinToken::new(win_token_addr, client.clone());
    let airvault = AirVault::new(airvault_addr, client.clone());

    subscribe_events(client, win_token.clone(), airvault.clone()).await;
}

async fn subscribe_events(
    client: Arc<Provider<Http>>,
    win_contract: WinToken<Provider<Http>>,
    airvault: AirVault<Provider<ethers::providers::Http>>,
) {
    // when are we targeting to do next airdrop?
    let mut next_airdrop = next_airdrop_block(client.get_block_number().await.unwrap());

    // make event stream that will recieve all events from airvault contract
    let events = airvault
        .events()
        .from_block(client.get_block_number().await.unwrap());
    let mut streams = events.stream().await.unwrap().take(1);

    // record all events into a container
    let mut events = Vec::<AirVaultEvents>::new();
    while let Some(Ok(evnt)) = streams.next().await {
        // do nothing, just add
        events.push(evnt);

        // check if we have to do airdrop
        let current_block: U64 = client.get_block_number().await.unwrap();
        if current_block >= next_airdrop {
            // if so proceed to reset the state and commit previous state rewards
            events = calculate_rewards(win_contract.clone(), events.clone(), current_block);
        }

        // re-set the airdrop target to end of next interval
        next_airdrop = next_airdrop_block(current_block + 1);
    }
}

// calculate rewards for this airdrop interval
// return deposit state for new interval
fn calculate_rewards(
    win_contract: WinToken<Provider<Http>>,
    events: Vec<AirVaultEvents>,
    rewarded_at: U64,
) -> Vec<AirVaultEvents> {
    // state for next interval to be returned. start empty
    let mut existing_rewards = HashMap::<Address, U256>::new();

    // mapping of each user rewards to be rewarded at last
    let mut rewards = HashMap::<Address, U256>::new();
    for event in events {
        match event {
            AirVaultEvents::DepositedFilter(deposited) => {
                let DepositedFilter {
                    account,
                    amount,
                    block_num,
                } = deposited;

                // save this reward amount to be counted in next reward cycle as well
                // if user have deposited, and we have not faced withdraw yet,
                // this amount is also valid and taken into account in next airdrop interval
                existing_rewards
                    .entry(account.clone())
                    .and_modify(|a| *a = a.saturating_add(amount))
                    .or_insert(amount);

                // this the power this amount have
                // this is equivalent to at what stage of interval had the deposited been made
                // amount multiplied by remaining blocks till next reward
                let power = amount.saturating_mul(
                    rewarded_at
                        .saturating_sub(block_num.as_u64().into())
                        .as_u64()
                        .into(),
                );

                // increae user reward balance by power
                rewards
                    .entry(account)
                    .and_modify(|r| {
                        *r = r.saturating_add(power);
                    })
                    .or_insert(power);
            }

            AirVaultEvents::WithdrawnFilter(withdrawn) => {
                let WithdrawnFilter {
                    account,
                    amount,
                    block_num,
                } = withdrawn;

                // since user withdrawn this,
                // this amount will be substracted for next airdrop interval
                existing_rewards
                    .entry(account.clone())
                    .and_modify(|a| *a = a.saturating_sub(amount));

                // power defincation
                let power = amount.saturating_mul(
                    rewarded_at
                        .saturating_sub(block_num.as_u64().into())
                        .as_u64()
                        .into(),
                );

                // substrate the power of this user by given amount
                rewards
                    .entry(account)
                    .and_modify(|r| {
                        *r = r.saturating_sub(power);
                    })
                    // ideally not possible. cannot withdraw that has never been deposited
                    // but can occur when we missed to run it before the user have had deposit made
                    .or_insert(0.into());
            }
        }
    }

    // iterate though all account and it's power
    // forward actual airdrop calls to another task
    for (account, reward) in rewards {
        let win_contract = win_contract.clone();
        tokio::task::spawn(async move {
            // we only do 5% of retained FUD
            // reward_amount = reward * 5 / 100
            let reward_amount = reward.saturating_mul(5.into()).checked_div(100.into());
            if let Some(reward_amount) = reward_amount {
                do_airdrop(win_contract, account, reward_amount).await;
            }
        });
    }

    // now we have a list of account and it's amount that is still be counted in next interval
    existing_rewards
        .into_iter()
        .filter_map(|(account, amount)| {
            if amount > 0.into() {
                // do no care to store 0 amount
                let block_num = rewarded_at.saturating_add(1.into()).as_u64().into();
                // treat all remaining amount as if that amount was freshly deposited
                // at start of upcoming airdrop interval
                let deposit_event = AirVaultEvents::DepositedFilter(DepositedFilter {
                    block_num,
                    account,
                    amount,
                });
                Some(deposit_event)
            } else {
                None
            }
        })
        .collect()
}

fn next_airdrop_block(current_block: U64) -> U64 {
    get_airdrop_interval() - (current_block % get_airdrop_interval()) + get_airdrop_interval()
}

// TODO:
// this can be configured from a fixed config file or also can be manipulation based on how
// many airdrop was done. For example increase the duration by x block every time a airdrop is
// sucessful. or raise it to infinity to do only certain rounds of airdrop
fn get_airdrop_interval() -> U64 {
    100.into()
}


// do the actual airdrop
// is just minting new win token in rewardee address
async fn do_airdrop(win_contract: WinToken<Provider<Http>>, address: H160, amount: U256) {
    let _signer = MINTER.1.parse::<LocalWallet>().unwrap();

    win_contract.mint(address, amount).send().await.unwrap();
}
