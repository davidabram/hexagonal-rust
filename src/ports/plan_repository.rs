use crate::domain::{Plan, PlanId};

pub trait PlanRepository: Send + Sync {
    async fn find_plan(&self, plan_id: &PlanId) -> Result<Option<Plan>, anyhow::Error>;
}
