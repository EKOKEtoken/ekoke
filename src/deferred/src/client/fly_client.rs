use async_trait::async_trait;
use candid::Principal;
use did::deferred::DeferredResult;
use did::fly::PicoFly;
use did::ID;

#[cfg(not(test))]
pub fn fly_client(principal: Principal) -> IcFlyClient {
    IcFlyClient { principal }
}

#[cfg(test)]
pub fn fly_client(_principal: Principal) -> IcFlyClient {
    IcFlyClient
}

#[async_trait]
pub trait FlyClient {
    /// Get contract reward. Returns $picoFly
    async fn get_contract_reward(
        &self,
        contract_id: ID,
        installments: u64,
    ) -> DeferredResult<PicoFly>;
}

#[cfg(not(test))]
/// Fly canister client
pub struct IcFlyClient {
    principal: Principal,
}

#[cfg(test)]
#[derive(Default)]
pub struct IcFlyClient;

#[cfg(not(test))]
#[async_trait]
impl FlyClient for IcFlyClient {
    /// Get contract reward. Returns $picoFly
    async fn get_contract_reward(
        &self,
        contract_id: ID,
        installments: u64,
    ) -> did::deferred::DeferredResult<PicoFly> {
        let result: (did::fly::FlyResult<PicoFly>,) = ic_cdk::call(
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
impl FlyClient for IcFlyClient {
    /// Get contract reward. Returns $picoFly
    async fn get_contract_reward(
        &self,
        _contract_id: ID,
        _installments: u64,
    ) -> DeferredResult<PicoFly> {
        Ok(71_000_u64.into())
    }
}
