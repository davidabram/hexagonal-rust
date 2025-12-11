use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::ports::{BillingProfileRepository, PlanRepository, SubscriptionRepository};
use crate::services::SubscriptionService;

use super::dtos::{CreateSubscriptionHttpBody, SubscriptionResponse};
use super::errors::ApiError;

#[derive(Clone)]
pub struct AppState<P, B, S>
where
    P: PlanRepository,
    B: BillingProfileRepository,
    S: SubscriptionRepository,
{
    pub subscription_service: Arc<SubscriptionService<P, B, S>>,
}

impl<P, B, S> AppState<P, B, S>
where
    P: PlanRepository,
    B: BillingProfileRepository,
    S: SubscriptionRepository,
{
    pub fn new(subscription_service: SubscriptionService<P, B, S>) -> Self {
        Self {
            subscription_service: Arc::new(subscription_service),
        }
    }
}

pub async fn create_subscription_handler<P, B, S>(
    State(state): State<AppState<P, B, S>>,
    Json(body): Json<CreateSubscriptionHttpBody>,
) -> Result<(StatusCode, Json<SubscriptionResponse>), ApiError>
where
    P: PlanRepository + 'static,
    B: BillingProfileRepository + 'static,
    S: SubscriptionRepository + 'static,
{
    let request = body.into_domain();

    let subscription = state
        .subscription_service
        .create_subscription(&request)
        .await
        .map_err(ApiError::from)?;

    let response = SubscriptionResponse::from(subscription);
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn health_check_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "ledgercloud",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
