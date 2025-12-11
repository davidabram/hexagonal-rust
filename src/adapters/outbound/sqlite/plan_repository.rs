use anyhow::Context;
use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::{Plan, PlanId};
use crate::ports::PlanRepository;

#[derive(Clone)]
pub struct SqlitePlanRepository {
    pool: SqlitePool,
}

impl SqlitePlanRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlanRepository for SqlitePlanRepository {
    async fn find_plan(&self, plan_id: &PlanId) -> Result<Option<Plan>, anyhow::Error> {
        let row = sqlx::query_as::<_, (String, String, i32, bool)>(
            "SELECT id, name, max_seats, requires_card_on_file FROM plans WHERE id = ?1",
        )
        .bind(plan_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch plan from database")?;

        Ok(
            row.map(|(id, name, max_seats, requires_card_on_file)| Plan {
                id: PlanId::new(id),
                name,
                max_seats: max_seats as u32,
                requires_card_on_file,
            }),
        )
    }
}
