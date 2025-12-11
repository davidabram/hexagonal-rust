use crate::domain::{
    CreateSubscriptionError, CreateSubscriptionRequest, Plan, Subscription, TenantId,
};
use crate::ports::{BillingProfileRepository, PlanRepository, SubscriptionRepository};

pub struct SubscriptionService<P, B, S>
where
    P: PlanRepository,
    B: BillingProfileRepository,
    S: SubscriptionRepository,
{
    plans: P,
    billing_profiles: B,
    subscriptions: S,
}

impl<P, B, S> SubscriptionService<P, B, S>
where
    P: PlanRepository,
    B: BillingProfileRepository,
    S: SubscriptionRepository,
{
    pub fn new(plans: P, billing_profiles: B, subscriptions: S) -> Self {
        Self {
            plans,
            billing_profiles,
            subscriptions,
        }
    }

    pub async fn create_subscription(
        &self,
        request: &CreateSubscriptionRequest,
    ) -> Result<Subscription, CreateSubscriptionError> {
        let plan = self
            .plans
            .find_plan(&request.plan_id)
            .await
            .map_err(CreateSubscriptionError::Unknown)?;

        let plan = match plan {
            Some(p) => p,
            None => {
                return Err(CreateSubscriptionError::PlanNotFound(
                    request.plan_id.clone(),
                ))
            }
        };

        if !self.tenant_allowed_on_plan(&request.tenant_id, &plan).await {
            return Err(CreateSubscriptionError::PlanNotAllowed(
                request.tenant_id.clone(),
                plan.id.clone(),
            ));
        }

        if plan.requires_card_on_file {
            let has_payment = self
                .billing_profiles
                .has_active_payment_method(&request.tenant_id)
                .await
                .map_err(CreateSubscriptionError::Unknown)?;

            if !has_payment {
                return Err(CreateSubscriptionError::MissingPaymentMethod(
                    request.tenant_id.clone(),
                ));
            }
        }

        let subscription = self
            .subscriptions
            .insert_subscription(&request.tenant_id, &request.plan_id)
            .await
            .map_err(CreateSubscriptionError::Unknown)?;

        Ok(subscription)
    }

    async fn tenant_allowed_on_plan(&self, _tenant_id: &TenantId, _plan: &Plan) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{PlanId, SubscriptionId};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockPlanRepository {
        plans: Arc<Mutex<Vec<Plan>>>,
    }

    impl MockPlanRepository {
        fn new() -> Self {
            Self {
                plans: Arc::new(Mutex::new(vec![
                    Plan {
                        id: PlanId("pro".to_string()),
                        name: "Pro Plan".to_string(),
                        max_seats: 10,
                        requires_card_on_file: true,
                    },
                    Plan {
                        id: PlanId("free".to_string()),
                        name: "Free Plan".to_string(),
                        max_seats: 1,
                        requires_card_on_file: false,
                    },
                ])),
            }
        }
    }

    #[async_trait]
    impl PlanRepository for MockPlanRepository {
        async fn find_plan(&self, plan_id: &PlanId) -> Result<Option<Plan>, anyhow::Error> {
            let plans = self.plans.lock().unwrap();
            Ok(plans.iter().find(|p| &p.id == plan_id).cloned())
        }
    }

    struct MockBillingProfileRepository {
        has_payment_method: bool,
    }

    #[async_trait]
    impl BillingProfileRepository for MockBillingProfileRepository {
        async fn has_active_payment_method(
            &self,
            _tenant_id: &TenantId,
        ) -> Result<bool, anyhow::Error> {
            Ok(self.has_payment_method)
        }
    }

    struct MockSubscriptionRepository;

    #[async_trait]
    impl SubscriptionRepository for MockSubscriptionRepository {
        async fn insert_subscription(
            &self,
            tenant_id: &TenantId,
            plan_id: &PlanId,
        ) -> Result<Subscription, anyhow::Error> {
            Ok(Subscription::new(
                SubscriptionId("sub_123".to_string()),
                tenant_id.clone(),
                plan_id.clone(),
            ))
        }
    }

    #[tokio::test]
    async fn test_create_subscription_success() {
        let service = SubscriptionService::new(
            MockPlanRepository::new(),
            MockBillingProfileRepository {
                has_payment_method: true,
            },
            MockSubscriptionRepository,
        );

        let request = CreateSubscriptionRequest {
            tenant_id: TenantId("tenant_1".to_string()),
            plan_id: PlanId("pro".to_string()),
        };

        let result = service.create_subscription(&request).await;
        assert!(result.is_ok());

        let subscription = result.unwrap();
        assert_eq!(subscription.tenant_id, request.tenant_id);
        assert_eq!(subscription.plan_id, request.plan_id);
    }

    #[tokio::test]
    async fn test_create_subscription_plan_not_found() {
        let service = SubscriptionService::new(
            MockPlanRepository::new(),
            MockBillingProfileRepository {
                has_payment_method: true,
            },
            MockSubscriptionRepository,
        );

        let request = CreateSubscriptionRequest {
            tenant_id: TenantId("tenant_1".to_string()),
            plan_id: PlanId("nonexistent".to_string()),
        };

        let result = service.create_subscription(&request).await;
        assert!(matches!(
            result,
            Err(CreateSubscriptionError::PlanNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_create_subscription_missing_payment_method() {
        let service = SubscriptionService::new(
            MockPlanRepository::new(),
            MockBillingProfileRepository {
                has_payment_method: false,
            },
            MockSubscriptionRepository,
        );

        let request = CreateSubscriptionRequest {
            tenant_id: TenantId("tenant_1".to_string()),
            plan_id: PlanId("pro".to_string()),
        };

        let result = service.create_subscription(&request).await;
        assert!(matches!(
            result,
            Err(CreateSubscriptionError::MissingPaymentMethod(_))
        ));
    }

    #[tokio::test]
    async fn test_create_subscription_free_plan_no_payment_required() {
        let service = SubscriptionService::new(
            MockPlanRepository::new(),
            MockBillingProfileRepository {
                has_payment_method: false,
            },
            MockSubscriptionRepository,
        );

        let request = CreateSubscriptionRequest {
            tenant_id: TenantId("tenant_1".to_string()),
            plan_id: PlanId("free".to_string()),
        };

        let result = service.create_subscription(&request).await;
        assert!(result.is_ok());
    }
}
