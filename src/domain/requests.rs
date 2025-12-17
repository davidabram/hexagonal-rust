use super::value_objects::{PlanId, TenantId};

#[derive(Debug, Clone)]
pub struct CreateSubscriptionRequest {
    pub tenant_id: TenantId,
    pub plan_id: PlanId,
}

