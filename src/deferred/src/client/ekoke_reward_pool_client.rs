use async_trait::async_trait;
use candid::Principal;
use did::deferred::DeferredResult;
use did::ekoke::Ekoke;
use did::ID;

#[cfg(not(test))]
pub fn ekoke_reward_pool_client(principal: Principal) -> IcEkokeRewardPoolClient {
    IcEkokeRewardPoolClient { principal }
}

#[cfg(test)]
pub fn ekoke_reward_pool_client(_principal: Principal) -> IcEkokeRewardPoolClient {
    IcEkokeRewardPoolClient
}

#[async_trait]
pub trait EkokeRewardPoolClient {
    /// Get contract reward. Returns $ekoke
    async fn get_contract_reward(
        &self,
        contract_id: ID,
        installments: u64,
    ) -> DeferredResult<Ekoke>;
}

#[cfg(not(test))]
/// Ekoke canister client
pub struct IcEkokeRewardPoolClient {
    principal: Principal,
}

#[cfg(test)]
#[derive(Default)]
pub struct IcEkokeRewardPoolClient;

#[cfg(not(test))]
#[async_trait]
impl EkokeRewardPoolClient for IcEkokeRewardPoolClient {
    /// Get contract reward. Returns $ekoke
    async fn get_contract_reward(
        &self,
        contract_id: ID,
        installments: u64,
    ) -> did::deferred::DeferredResult<Ekoke> {
        let result: (did::ekoke::EkokeResult<Ekoke>,) = ic_cdk::call(
            self.principal,
            "get_contract_reward",
            (contract_id, installments),
        )
        .await
        .map_err(|(code, err)| did::deferred::DeferredError::CanisterCall(code, err))?;

        let reward = result.0?;
        Ok(reward)
    }
}

#[cfg(test)]
#[async_trait]
impl EkokeRewardPoolClient for IcEkokeRewardPoolClient {
    /// Get contract reward. Returns $ekoke
    async fn get_contract_reward(
        &self,
        _contract_id: ID,
        _installments: u64,
    ) -> DeferredResult<Ekoke> {
        Ok(71_000_u64.into())
    }
}
