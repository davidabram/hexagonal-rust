use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::value_objects::{PlanId, SubscriptionId, TenantId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub name: String,
    pub max_seats: u32,
    pub requires_card_on_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub tenant_id: TenantId,
    pub plan_id: PlanId,
    pub created_at: DateTime<Utc>,
}

impl Subscription {
    pub fn new(id: SubscriptionId, tenant_id: TenantId, plan_id: PlanId) -> Self {
        Self {
            id,
            tenant_id,
            plan_id,
            created_at: Utc::now(),
        }
    }
}

