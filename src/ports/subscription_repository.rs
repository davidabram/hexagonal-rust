use crate::domain::{PlanId, Subscription, TenantId};

pub trait SubscriptionRepository: Send + Sync {
    async fn insert_subscription(
        &self,
        tenant_id: &TenantId,
        plan_id: &PlanId,
    ) -> Result<Subscription, anyhow::Error>;
}
