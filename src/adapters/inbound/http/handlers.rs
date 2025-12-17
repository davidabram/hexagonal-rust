use axum::{extract::State, http::StatusCode, Json};
use opentelemetry::trace::Status;
use std::sync::Arc;
use tracing::{info, instrument, Span};

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

#[instrument(
    name = "create_subscription_handler",
    skip(state, body),
    fields(
        tenant_id = %body.tenant_id,
        plan_id = %body.plan_id,
    )
)]
pub async fn create_subscription_handler<P, B, S>(
    State(state): State<AppState<P, B, S>>,
    Json(body): Json<CreateSubscriptionHttpBody>,
) -> Result<(StatusCode, Json<SubscriptionResponse>), ApiError>
where
    P: PlanRepository + 'static,
    B: BillingProfileRepository + 'static,
    S: SubscriptionRepository + 'static,
{
    let request = body.into();

    let subscription = state
        .subscription_service
        .create_subscription(&request)
        .await
        .map_err(ApiError::from)?;

    info!(
        subscription_id = %subscription.id,
        tenant_id = %subscription.tenant_id,
        plan_id = %subscription.plan_id,
        "subscription created successfully"
    );

    let span = Span::current();
    span.record("http.response.status_code", 201);
    span.record("subscription_id", subscription.id.to_string().as_str());

    opentelemetry::trace::get_active_span(|span| {
        span.set_status(Status::Ok);
    });

    let response = SubscriptionResponse::from(subscription);
    Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(name = "health_check_handler")]
pub async fn health_check_handler() -> Json<serde_json::Value> {
    opentelemetry::trace::get_active_span(|span| {
        span.set_status(Status::Ok);
    });

    Json(serde_json::json!({
        "status": "healthy",
        "service": "ledgercloud",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
