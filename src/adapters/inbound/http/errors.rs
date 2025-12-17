use axum::{http::StatusCode, response::IntoResponse, Json};
use opentelemetry::trace::Status;
use serde::Serialize;
use std::collections::HashMap;
use tracing::{error, warn, Span};

use crate::domain::CreateSubscriptionError;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
    pub code: u16,
    #[serde(skip)]
    pub error_type: Option<String>,
    #[serde(skip)]
    pub error_attributes: HashMap<String, String>,
}

impl From<CreateSubscriptionError> for ApiError {
    fn from(e: CreateSubscriptionError) -> Self {
        match &e {
            CreateSubscriptionError::PlanNotFound(plan_id) => {
                warn!(
                    error = %e,
                    plan_id = %plan_id,
                    "plan not found"
                );
                let mut attrs = HashMap::new();
                attrs.insert("plan_id".to_string(), plan_id.to_string());
                ApiError {
                    message: format!("Plan {} not found", plan_id),
                    code: 404,
                    error_type: Some("PlanNotFound".to_string()),
                    error_attributes: attrs,
                }
            }
            CreateSubscriptionError::PlanNotAllowed(tenant_id, plan_id) => {
                warn!(
                    error = %e,
                    tenant_id = %tenant_id,
                    plan_id = %plan_id,
                    "tenant not allowed on plan"
                );
                let mut attrs = HashMap::new();
                attrs.insert("tenant_id".to_string(), tenant_id.to_string());
                attrs.insert("plan_id".to_string(), plan_id.to_string());
                ApiError {
                    message: format!("Tenant {} is not allowed on plan {}", tenant_id, plan_id),
                    code: 403,
                    error_type: Some("PlanNotAllowed".to_string()),
                    error_attributes: attrs,
                }
            }
            CreateSubscriptionError::MissingPaymentMethod(tenant_id) => {
                warn!(
                    error = %e,
                    tenant_id = %tenant_id,
                    "missing payment method"
                );
                let mut attrs = HashMap::new();
                attrs.insert("tenant_id".to_string(), tenant_id.to_string());
                ApiError {
                    message: format!("Tenant {} has no active payment method on file", tenant_id),
                    code: 422,
                    error_type: Some("MissingPaymentMethod".to_string()),
                    error_attributes: attrs,
                }
            }
            CreateSubscriptionError::Unexpected(source) => {
                error!(
                    error = %source,
                    "unexpected error during subscription creation"
                );
                ApiError {
                    message: "Internal server error".into(),
                    code: 500,
                    error_type: Some("Unexpected".to_string()),
                    error_attributes: HashMap::new(),
                }
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let current_span = Span::current();

        opentelemetry::trace::get_active_span(|span| {
            span.set_status(Status::error(self.message.clone()));
        });

        current_span.record("http.response.status_code", self.code);

        if let Some(error_type) = &self.error_type {
            current_span.record("error.type", error_type.as_str());
        }

        for (key, value) in &self.error_attributes {
            current_span.record(key.as_str(), value.as_str());
        }

        (status, Json(self)).into_response()
    }
}
