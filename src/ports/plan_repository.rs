use async_trait::async_trait;

use crate::domain::{Plan, PlanId};

#[async_trait]
pub trait PlanRepository: Send + Sync {
    async fn find_plan(&self, plan_id: &PlanId) -> Result<Option<Plan>, anyhow::Error>;
}
