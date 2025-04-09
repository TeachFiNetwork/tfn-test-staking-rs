<p align="center">
  <a href="https://teachfi.network/" target="blank"><img src="https://teachfi.network/teachfi-logo.svg" width="256" alt="TeachFi Logo" /><br/>Test DEX</a>
</p>
<br/>
<br/>
<br/>

# Description

This is a child contract of Platform SC. A separate instance is deployed for each platform subscriber.\
Interest earning while helping students secure their blockchain projects. Users whitelisted in the parent Platform SC can create staking pools for any token and provide rewards in order to incentivize other students to lock the respective assets. Another great tool for helping students develop financial literacy.
<br/>
<br/>
<br/>
## Endpoints

<br/>

```rust
createStake(
    stake_type: StakeType,
    token: TokenIdentifier,
    token_decimals: u8,
    reward_token: TokenIdentifier,
)
```
>[!IMPORTANT]
>*Requirements:* state = active.

>[!NOTE]
>Creates a new staking pool for `token` with rewards in `reward_token`. The `stake_type` can be selected from FixedAPR or DynamicAPR.

>[!WARNING]
>The transaction should have a 0.05 eGLD value, needed to issue a Liquid token for the newly created pool.
<br/>

```rust
setStakeActive(id: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pool owner.

>[!NOTE]
>Activates staking in the pool specified by the `id` parameter.
<br/>

```rust
setStakeInactive(id: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pool owner.

>[!NOTE]
>Disables staking in the pool specified by the `id` parameter.
<br/>

```rust
setStakeRewardsPerSecond(id: u64, rewards_per_second: BigUint)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pool owner.

>[!NOTE]
>Applicable only to FixedAPR pools, represents the amount of reward tokens a user receives every second for each staked token. 
>APR formula: 100 * rewards_per_second * 60 * 60 * 24 * 365 * reward_token_price / stake_token_price.
<br/>

```rust
setStakeEndTime(id: u64, new_end_time: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pool owner.

>[!NOTE]
>Sets a new `end_time` for the pool specified by the `id` parameter.
<br/>

```rust
depositRewards(id: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active.

>[!NOTE]
>Adds more reward tokens to the pool identified by the `id` parameter. If the stake type is DynamicAPR, its APR automatically increases as its formula is: 
>100 * reward_tokens * reward_token_price * 60 * 60 * 24 * 365 / (end_time - current_time) / (staked_tokens * stake_token_price)
<br/>

```rust
withdrawRewards(id: u64, opt_amount: BigUint)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pool owner.

>[!NOTE]
>Withdraws `opt_amount` of reward tokens if specified, and all of them if not.

>[!WARNING]
>Withdrawing all the rewards, automatically renders the pool inactive.
<br/>

```rust
changeStakeType(id: u64, new_stake_type: StakeType)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pool owner, pool_state = inactive.

>[!NOTE]
>Changes the stake type to the new specified value.

>[!WARNING]
>This operation might have undesired effects. Make sure to calculate the formula changes before.
<br/>

```rust
userStake()
```
>[!IMPORTANT]
>*Requirements:* state = active, pool_state = active.

>[!NOTE]
>The pool is identified by the payment token as the stake token. The SC mints payment_amount Liquid tokens and sends them to the caller.
<br/>

```rust
userUnstake()
```
>[!IMPORTANT]
>*Requirements:* state = active, pool_state = active.

>[!NOTE]
>The pool is identified by the payment token as the liquid token. The SC burns the received tokens and sends back to the caller an equal amount of stake tokens along with the earned rewards.
<br/>

```rust
claimRewards()
```
>[!IMPORTANT]
>*Requirements:* state = active, pool_state = active.

>[!NOTE]
>The pool is identified by the payment token as the liquid token. The SC computes the amount of earned rewards and sends them back to the caller along with the liquid tokens.
<br/>

```rust
setStateActive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as active.
<br/>

```rust
setStateInactive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as inactive.
<br/>

```rust
setPlatformAddress(platform_sc: ManagedAddress)
```
>[!IMPORTANT]
>*Requirements:* caller = owner, platform should be empty.

>[!NOTE]
>Sets the Platform SC address and retrieves the governance token id from it.

<br/>

## View functions

```rust
getState() -> State
```
>Returns the state of the SC (Active or Inactive).
<br/>

```rust
getPlatformAddress() -> ManagedAddress
```
>Returns the Platform SC address if set.
<br/>

```rust
getStake(id: u64) -> Stake
```
>Returns the Stake object identified by the `id` parameter.
<br/>

```rust
getStakes() -> ManagedVec<Stake>
```
>Returns all the pools.
<br/>

```rust
getStakeByToken(token: TokenIdentifier) -> Option<Stake>
```
>If a pool with `stake_token` = `token` is found, it returns Some(pool) and None otherwise.
<br/>

```rust
getStakeByLiquidToken(token: TokenIdentifier) -> Option<Stake>
```
>If a pool with `liquid_token` = `token` is found, it returns Some(pool) and None otherwise.
<br/>

```rust
getUserRewards(id: u64, staked_tokens: ManagedVec<EsdtTokenPayment>) -> BigUint
```
>Returns the amount of rewards earned by a user in the pool identifier by the `id` parameter, for the liquid tokens specified in `staked_tokens`.

<br/>

## Custom types

```rust
pub enum State {
    Inactive,
    Active,
}
```

<br/>

```rust
pub enum StakeType {
    FixedAPR,
    DynamicAPR,
}
```

<br/>

```rust
pub struct Stake<M: ManagedTypeApi> {
    pub id: u64,
    pub owner: ManagedAddress<M>,
    pub state: State,
    pub stake_type: StakeType,
    pub token: TokenIdentifier<M>,
    pub token_decimals: u8,
    pub liquid_token: TokenIdentifier<M>,
    pub reward_token: TokenIdentifier<M>,
    pub staked_amount: BigUint<M>,
    pub rewards_amount: BigUint<M>,
    pub claimable_rewards: BigUint<M>,
    pub remaining_rewards: BigUint<M>,
    pub rewards_per_second: BigUint<M>, // apr
    pub start_time: u64,
    pub end_time: u64,
    pub remaining_time: u64,
    pub rps: BigUint<M>,
    pub last_rps_update_time: u64,
}
```

<br/>

```rust
pub struct StakeTokenAttributes<M: ManagedTypeApi> {
    pub rps: BigUint<M>,
}
```
