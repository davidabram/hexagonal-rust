#![allow(dead_code)]

use anyhow::Context;
use serde::Serialize;
use tracing::{error, instrument};

#[derive(Serialize)]
struct CreateCustomerRequest<'a> {
    email: &'a str,
}

#[derive(Serialize)]
struct AddPaymentMethodRequest<'a> {
    customer_id: &'a str,
    token: &'a str,
}

#[allow(dead_code)]
pub struct PaymentClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl PaymentClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    #[instrument(
        name = "payment_create_customer",
        skip(self),
        fields(
            http.method = "POST",
            http.url = %format!("{}/customers", self.base_url),
            customer.email = %email
        )
    )]
    pub async fn create_customer(&self, email: &str) -> Result<String, anyhow::Error> {
        let url = format!("{}/customers", self.base_url);

        let request = CreateCustomerRequest { email };

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .context("failed to call payment provider /customers endpoint")
            .inspect_err(|e| {
                error!(error = %e, email = %email, "payment provider /customers request failed");
            })?;

        let response = response
            .error_for_status()
            .context("payment provider returned error status")
            .inspect_err(|e| {
                error!(error = %e, "payment provider returned error status");
            })?;

        let body: serde_json::Value = response
            .json()
            .await
            .context("failed to parse payment provider response JSON")
            .inspect_err(|e| {
                error!(error = %e, "failed to parse payment provider JSON response");
            })?;

        let id = body
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("payment provider response missing `id` field"))
            .inspect_err(|e| {
                error!(error = %e, "payment provider response missing `id` field");
            })?;

        Ok(id.to_string())
    }

    #[instrument(
        name = "payment_add_payment_method",
        skip(self, payment_token),
        fields(
            http.method = "POST",
            http.url = %format!("{}/payment_methods", self.base_url),
            customer_id = %customer_id
        )
    )]
    pub async fn add_payment_method(
        &self,
        customer_id: &str,
        payment_token: &str,
    ) -> Result<String, anyhow::Error> {
        let url = format!("{}/payment_methods", self.base_url);

        let request = AddPaymentMethodRequest {
            customer_id,
            token: payment_token,
        };

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .context("failed to call payment provider /payment_methods endpoint")
            .inspect_err(|e| {
                error!(error = %e, customer_id = %customer_id, "payment provider /payment_methods request failed");
            })?;

        let response = response
            .error_for_status()
            .context("payment provider returned error status")
            .inspect_err(|e| {
                error!(error = %e, "payment provider returned error status");
            })?;

        let body: serde_json::Value = response
            .json()
            .await
            .context("failed to parse payment provider response JSON")
            .inspect_err(|e| {
                error!(error = %e, "failed to parse payment provider JSON response");
            })?;

        let id = body
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("payment provider response missing `id` field"))
            .inspect_err(|e| {
                error!(error = %e, "payment provider response missing `id` field");
            })?;

        Ok(id.to_string())
    }
}
