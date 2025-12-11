#![allow(dead_code)]

use anyhow::Context;

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

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({ "email": email }))
            .send()
            .await
            .context("failed to call payment provider /customers endpoint")?;

        if !response.status().is_success() {
            anyhow::bail!("payment provider returned status {}", response.status());
        }

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

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "customer_id": customer_id,
                "token": payment_token
            }))
            .send()
            .await
            .context("failed to call payment provider /payment_methods endpoint")?;

        if !response.status().is_success() {
            anyhow::bail!("payment provider returned status {}", response.status());
        }

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
