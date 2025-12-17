use anyhow::Context;
use sqlx::SqlitePool;
use tracing::{error, instrument};

use crate::domain::{Plan, PlanId};
use crate::ports::PlanRepository;

struct PlanRow {
    id: String,
    name: String,
    max_seats: i64,
    requires_card_on_file: bool,
}

impl From<PlanRow> for Plan {
    fn from(row: PlanRow) -> Self {
        Self {
            id: PlanId::new(row.id),
            name: row.name,
            max_seats: row.max_seats as u32,
            requires_card_on_file: row.requires_card_on_file,
        }
    }
}

#[derive(Clone)]
pub struct SqlitePlanRepository {
    pool: SqlitePool,
}

impl SqlitePlanRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl PlanRepository for SqlitePlanRepository {
    #[instrument(
        name = "find_plan",
        skip(self),
        fields(db.system = "sqlite", plan_id = %plan_id)
    )]
    async fn find_plan(&self, plan_id: &PlanId) -> Result<Option<Plan>, anyhow::Error> {
        let plan_id_str = plan_id.as_ref();
        let row = sqlx::query_as!(
            PlanRow,
            r#"SELECT id as "id!", name as "name!", max_seats as "max_seats!", requires_card_on_file as "requires_card_on_file!" FROM plans WHERE id = ?1"#,
            plan_id_str
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to fetch plan from database")
        .inspect_err(|e| {
            error!(error = %e, plan_id = %plan_id, "plan query failed");
        })?;

        Ok(row.map(Into::into))
    }
}
