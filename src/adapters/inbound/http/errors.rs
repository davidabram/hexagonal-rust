#![allow(dead_code)]

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::domain::CreateSubscriptionError;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
    pub code: u16,
}

#[allow(dead_code)]
impl ApiError {
    #[allow(dead_code)]
    pub fn new(message: impl Into<String>, code: u16) -> Self {
        Self {
            message: message.into(),
            code,
        }
    }
}

impl From<CreateSubscriptionError> for ApiError {
    fn from(e: CreateSubscriptionError) -> Self {
        match e {
            CreateSubscriptionError::PlanNotFound(plan_id) => ApiError {
                message: format!("Plan {} not found", plan_id),
                code: 404,
            },
            CreateSubscriptionError::PlanNotAllowed(tenant_id, plan_id) => ApiError {
                message: format!("Tenant {} is not allowed on plan {}", tenant_id, plan_id),
                code: 403,
            },
            CreateSubscriptionError::MissingPaymentMethod(tenant_id) => ApiError {
                message: format!("Tenant {} has no active payment method on file", tenant_id),
                code: 422,
            },
            CreateSubscriptionError::Unexpected(_source) => ApiError {
                message: "Internal server error".into(),
                code: 500,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}
