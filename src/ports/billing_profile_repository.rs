use crate::domain::TenantId;

pub trait BillingProfileRepository: Send + Sync {
    async fn has_active_payment_method(&self, tenant_id: &TenantId) -> Result<bool, anyhow::Error>;
}
