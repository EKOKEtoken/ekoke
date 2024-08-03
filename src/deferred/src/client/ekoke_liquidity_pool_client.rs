use std::collections::HashMap;

use async_trait::async_trait;
use candid::{Nat, Principal};
use did::deferred::DeferredResult;

#[cfg(not(test))]
pub fn ekoke_liquidity_pool_client(principal: Principal) -> IcEkokeLiquidityPoolClient {
    IcEkokeLiquidityPoolClient { principal }
}

#[cfg(test)]
pub fn ekoke_liquidity_pool_client(_principal: Principal) -> IcEkokeLiquidityPoolClient {
    IcEkokeLiquidityPoolClient
}

#[async_trait]
pub trait EkokeLiquidityPoolClient {
    /// Refund investors
    async fn refund_investors(&self, refunds: HashMap<Principal, Nat>) -> DeferredResult<()>;
}

#[cfg(not(test))]
/// Ekoke canister client
pub struct IcEkokeLiquidityPoolClient {
    principal: Principal,
}

#[cfg(test)]
#[derive(Default)]
pub struct IcEkokeLiquidityPoolClient;

#[cfg(not(test))]
#[async_trait]
impl EkokeLiquidityPoolClient for IcEkokeLiquidityPoolClient {
    /// Get contract reward. Returns $ekoke
    async fn refund_investors(
        &self,
        refunds: HashMap<Principal, Nat>,
    ) -> did::deferred::DeferredResult<()> {
        let result: (Result<(), icrc::icrc1::transfer::TransferError>,) =
            ic_cdk::call(self.principal, "refund_investors", (refunds,))
                .await
                .map_err(|(code, err)| did::deferred::DeferredError::CanisterCall(code, err))?;

        result.0.map_err(|err| {
            did::deferred::DeferredError::CloseContract(
                did::deferred::CloseContractError::RefundInvestors(err),
            )
        })?;
        Ok(())
    }
}

#[cfg(test)]
#[async_trait]
impl EkokeLiquidityPoolClient for IcEkokeLiquidityPoolClient {
    /// Get contract reward. Returns $ekoke
    async fn refund_investors(&self, _refunds: HashMap<Principal, Nat>) -> DeferredResult<()> {
        Ok(())
    }
}
