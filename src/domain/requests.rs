use super::value_objects::{PlanId, TenantId};

#[derive(Debug, Clone)]
pub struct CreateSubscriptionRequest {
    pub tenant_id: TenantId,
    pub plan_id: PlanId,
}

impl CreateSubscriptionRequest {
    pub fn new(tenant_id: TenantId, plan_id: PlanId) -> Self {
        Self { tenant_id, plan_id }
    }
}
