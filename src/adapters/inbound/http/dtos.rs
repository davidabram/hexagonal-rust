use serde::{Deserialize, Serialize};

use crate::domain::{CreateSubscriptionRequest, PlanId, Subscription, TenantId};

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionHttpBody {
    pub tenant_id: String,
    pub plan_id: String,
}

impl CreateSubscriptionHttpBody {
    pub fn into_domain(self) -> CreateSubscriptionRequest {
        CreateSubscriptionRequest {
            tenant_id: TenantId::new(self.tenant_id),
            plan_id: PlanId::new(self.plan_id),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub tenant_id: String,
    pub plan_id: String,
    pub created_at: String,
}

impl From<Subscription> for SubscriptionResponse {
    fn from(s: Subscription) -> Self {
        Self {
            id: s.id.as_str().to_string(),
            tenant_id: s.tenant_id.as_str().to_string(),
            plan_id: s.plan_id.as_str().to_string(),
            created_at: s.created_at.to_rfc3339(),
        }
    }
}
