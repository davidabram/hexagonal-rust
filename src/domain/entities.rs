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

impl Plan {
    pub fn new(id: PlanId, name: String, max_seats: u32, requires_card_on_file: bool) -> Self {
        Self {
            id,
            name,
            max_seats,
            requires_card_on_file,
        }
    }

    pub fn requires_payment(&self) -> bool {
        self.requires_card_on_file
    }

    pub fn is_within_seat_limit(&self, seats: u32) -> bool {
        seats <= self.max_seats
    }
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

    pub fn with_created_at(
        id: SubscriptionId,
        tenant_id: TenantId,
        plan_id: PlanId,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            tenant_id,
            plan_id,
            created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingProfile {
    pub tenant_id: TenantId,
    pub has_active_payment_method: bool,
    pub payment_provider_customer_id: Option<String>,
}

impl BillingProfile {
    pub fn new(tenant_id: TenantId, has_active_payment_method: bool) -> Self {
        Self {
            tenant_id,
            has_active_payment_method,
            payment_provider_customer_id: None,
        }
    }

    pub fn with_payment_provider_id(
        tenant_id: TenantId,
        payment_provider_customer_id: String,
    ) -> Self {
        Self {
            tenant_id,
            has_active_payment_method: true,
            payment_provider_customer_id: Some(payment_provider_customer_id),
        }
    }
}
