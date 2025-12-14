#![allow(dead_code)]

use anyhow::Context;
use serde::Serialize;

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
            .context("failed to call payment provider /customers endpoint")?
            .error_for_status()
            .context("payment provider returned error status")?;

        let body: serde_json::Value = response
            .json()
            .await
            .context("failed to parse payment provider response JSON")?;

        let id = body
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("payment provider response missing `id` field"))?;

        Ok(id.to_string())
    }

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
            .context("failed to call payment provider /payment_methods endpoint")?
            .error_for_status()
            .context("payment provider returned error status")?;

        let body: serde_json::Value = response
            .json()
            .await
            .context("failed to parse payment provider response JSON")?;

        let id = body
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("payment provider response missing `id` field"))?;

        Ok(id.to_string())
    }
}
