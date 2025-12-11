use thiserror::Error;

use super::value_objects::{PlanId, TenantId};

#[derive(Debug, Error)]
pub enum CreateSubscriptionError {
    #[error("plan {0} does not exist")]
    PlanNotFound(PlanId),

    #[error("tenant {0} is not allowed on plan {1}")]
    PlanNotAllowed(TenantId, PlanId),

    #[error("tenant {0} has no active payment method")]
    MissingPaymentMethod(TenantId),

    #[error("an unexpected error occurred")]
    Unknown(#[source] anyhow::Error),
}

impl CreateSubscriptionError {
    pub fn unknown(error: impl Into<anyhow::Error>) -> Self {
        Self::Unknown(error.into())
    }
}
