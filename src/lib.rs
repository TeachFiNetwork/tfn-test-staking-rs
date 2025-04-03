#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod helpers;
pub mod user;

use common::{errors::*, storage::*};
use tfn_platform::common::config::ProxyTrait as _;
use tfn_staking::common::{errors::*, consts::*};

#[multiversx_sc::contract]
pub trait TFNTestStakingContract<ContractReader>:
common::config::ConfigModule
+common::storage::StorageModule
+helpers::HelpersModule
{
    #[init]
    fn init(&self) {
        let caller = self.blockchain().get_caller();
        if self.blockchain().is_smart_contract(&caller) {
            self.platform_sc().set(caller);
            self.set_state_active();
        }
    }

    #[upgrade]
    fn upgrade(&self) {
    }

    #[payable("EGLD")]
    #[endpoint(createStake)]
    fn create_stake(
        &self,
        stake_type: StakeType,
        token: TokenIdentifier,
        token_decimals: u8,
        reward_token: TokenIdentifier,
    ) {
        let caller = self.blockchain().get_caller();
        self.check_whitelisted(&caller);
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(self.get_stake_by_token(&token).is_none(), ERROR_STAKE_EXISTS);

        let issue_cost = self.call_value().egld_value().clone_value();
        let token_display_name = ManagedBuffer::from(STAKE_TOKEN_NAME_PREFIX).concat(token.ticker());
        let token_ticker = ManagedBuffer::from(STAKE_TOKEN_TICKER_PREFIX).concat(token.ticker());
        self.send()
            .esdt_system_sc_proxy()
            .issue_and_set_all_roles(
                issue_cost,
                token_display_name,
                token_ticker,
                EsdtTokenType::Meta,
                DEFAULT_STAKE_TOKEN_DECIMALS,
            )
            .with_callback(self.callbacks().stake_token_issue_callback(&caller, stake_type, token, token_decimals, reward_token))
            .async_call_and_exit();
    }

    #[callback]
    fn stake_token_issue_callback(
        &self,
        caller: &ManagedAddress,
        stake_type: StakeType,
        token: TokenIdentifier,
        token_decimals: u8,
        reward_token: TokenIdentifier,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(stake_token) => {
                let id = self.last_stake_id().get();
                let stake = Stake {
                    id,
                    owner: caller.clone(),
                    state: State::Inactive,
                    stake_type,
                    token,
                    token_decimals,
                    liquid_token: stake_token,
                    reward_token,
                    staked_amount: BigUint::zero(),
                    rewards_amount: BigUint::zero(),
                    claimable_rewards: BigUint::zero(),
                    remaining_rewards: BigUint::zero(),
                    rewards_per_second: BigUint::zero(),
                    start_time: 0,
                    end_time: 0,
                    remaining_time: 0,
                    rps: BigUint::zero(),
                    last_rps_update_time: 0,
                };
                self.stake(id).set(stake);
                self.last_stake_id().set(id + 1);
            }
            ManagedAsyncCallResult::Err(_) => {
                let issue_cost = self.call_value().egld_value();
                self.send().direct_egld(caller, &issue_cost);
            }
        }
    }

    #[endpoint(setStakeActive)]
    fn set_stake_active(&self, id: u64) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);

        let mut stake = self.stake(id).get();
        require!(stake.owner == self.blockchain().get_caller(), ERROR_NOT_STAKE_OWNER);

        require!(stake.rewards_per_second > 0, ERROR_ZERO_APR);
        require!(stake.rewards_amount > 0, ERROR_NO_REWARDS);
        require!(stake.end_time > self.blockchain().get_block_timestamp(), ERROR_STAKE_EXPIRED);

        stake.state = State::Active;
        self.stake(id).set(stake);
    }

    #[endpoint(setStakeInactive)]
    fn set_stake_inactive(&self, id: u64) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);

        let mut stake = self.stake(id).get();
        require!(stake.owner == self.blockchain().get_caller(), ERROR_NOT_STAKE_OWNER);

        stake.state = State::Inactive;
        self.stake(id).set(stake);
    }

    #[endpoint(setStakeRewardsPerSecond)]
    fn set_stake_rewards_per_second(&self, id: u64, rewards_per_second: BigUint) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);
        require!(rewards_per_second > 0, ERROR_ZERO_APR);

        let mut stake = self.stake(id).get();
        require!(stake.owner == self.blockchain().get_caller(), ERROR_NOT_STAKE_OWNER);

        self.update_rps(&mut stake);
        stake.rewards_per_second = rewards_per_second;
        self.stake(id).set(stake);
    }

    #[endpoint(setStakeEndTime)]
    fn set_stake_end_time(&self, id: u64, new_end_time: u64) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);
        require!(new_end_time > self.blockchain().get_block_timestamp(), ERROR_STAKE_EXPIRED);

        let mut stake = self.stake(id).get();
        require!(stake.owner == self.blockchain().get_caller(), ERROR_NOT_STAKE_OWNER);

        self.update_rps(&mut stake);
        if stake.start_time > 0 && stake.end_time > 0 {
            stake.remaining_time += new_end_time;
            stake.remaining_time -= stake.end_time;
        }
        stake.end_time = new_end_time;
        self.stake(id).set(stake);
    }

    #[payable("*")]
    #[endpoint(depositRewards)]
    fn deposit_rewards(&self, id: u64) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);

        let mut stake = self.stake(id).get();
        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == stake.reward_token, ERROR_WRONG_PAYMENT_TOKEN);

        self.update_rps(&mut stake);
        stake.rewards_amount += &payment.amount;
        stake.remaining_rewards += payment.amount;
        self.stake(id).set(stake);
    }

    #[endpoint(withdrawRewards)]
    fn withdraw_rewards(&self, id: u64, opt_amount: OptionalValue<BigUint>) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);

        let mut stake = self.stake(id).get();
        require!(stake.owner == self.blockchain().get_caller(), ERROR_NOT_STAKE_OWNER);

        self.update_rps(&mut stake);
        let amount = match opt_amount {
            OptionalValue::Some(amount) => amount,
            OptionalValue::None => stake.remaining_rewards.clone(),
        };
        require!(stake.remaining_rewards >= amount, ERROR_HIGH_AMOUNT);

        self.send().direct_esdt(&self.blockchain().get_caller(), &stake.reward_token, 0, &amount);
        stake.rewards_amount -= &amount;
        stake.remaining_rewards -= &amount;
        self.stake(id).set(stake);
    }

    #[endpoint(changeStakeType)]
    fn change_stake_type(&self, id: u64, new_stake_type: StakeType) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);

        let mut stake = self.stake(id).get();
        require!(stake.owner == self.blockchain().get_caller(), ERROR_NOT_STAKE_OWNER);
        require!(stake.state == State::Inactive, ERROR_STAKE_ACTIVE);

        stake.stake_type = new_stake_type;
        self.stake(id).set(stake);
    }

    // helpers
    fn check_whitelisted(&self, address: &ManagedAddress) {
        self.platform_contract_proxy()
            .contract(self.platform_sc().get())
            .check_whitelisted(address)
            .execute_on_dest_context::<()>();
    }

    // proxies
    #[proxy]
    fn platform_contract_proxy(&self) -> tfn_platform::Proxy<Self::Api>;
}
