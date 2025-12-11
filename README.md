# LedgerCloud - Hexagonal Architecture B2B SaaS

A demonstration of hexagonal architecture (ports and adapters) in Rust, implementing a multi-tenant B2B SaaS subscription billing system with robust error handling.

## Error Handling Strategy

This project demonstrates proper error handling in a hexagonal architecture:

- **Infrastructure Layer**: Uses `anyhow::Error` with `.context()` for enrichment
- **Ports**: Return `Result<T, anyhow::Error>` - the boundary between infrastructure and domain
- **Services**: Convert `anyhow::Error` to domain errors using `.map_err()`
- **Domain Errors**: Use `thiserror` for semantic, matchable enums
- **HTTP Layer**: Convert domain errors to API errors with appropriate HTTP status codes

### Error Flow Example

```rust
// Infrastructure (Repository)
async fn find_plan(&self, plan_id: &PlanId) -> Result<Option<Plan>, anyhow::Error> {
    sqlx::query!(...)
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch plan from database")?  // anyhow context
}

// Service Layer
let plan = self.plans
    .find_plan(&request.plan_id)
    .await
    .map_err(CreateSubscriptionError::Unknown)?;  // Convert to domain error

// HTTP Layer
CreateSubscriptionError::PlanNotFound(id) => ApiError {
    message: format!("Plan {} not found", id),
    code: 404,  // Map to HTTP status
}
```

## Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- Rust 1.75 or later (provided by Nix)
- SQLite (provided by Nix)

## Quick Start

```bash
cp .env.example .env

nix develop

sqlx migrate run

cargo run
```

The API will be available at `http://localhost:3000`

### Create Test Subscriptions

```bash
curl -X POST http://localhost:3000/api/subscriptions \
  -H "Content-Type: application/json" \
  -d '{"tenant_id": "tenant_no_payment", "plan_id": "pro"}'

curl -X POST http://localhost:3000/api/subscriptions \
  -H "Content-Type: application/json" \
  -d '{"tenant_id": "tenant_with_payment", "plan_id": "pro"}'

curl -X POST http://localhost:3000/api/subscriptions \
  -H "Content-Type: application/json" \
  -d '{"tenant_id": "tenant_payment_expired", "plan_id": "enterprise"}'

curl -X POST http://localhost:3000/api/subscriptions \
  -H "Content-Type: application/json" \
  -d '{"tenant_id": "tenant_free_plan", "plan_id": "free"}'
```

