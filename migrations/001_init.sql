CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    max_seats INTEGER NOT NULL,
    requires_card_on_file BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS billing_profiles (
    tenant_id TEXT PRIMARY KEY,
    has_active_payment_method BOOLEAN NOT NULL DEFAULT FALSE,
    payment_provider_customer_id TEXT
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    plan_id TEXT NOT NULL REFERENCES plans(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_tenant_id ON subscriptions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_plan_id ON subscriptions(plan_id);

INSERT OR IGNORE INTO plans (id, name, max_seats, requires_card_on_file) VALUES
    ('free', 'Free Plan', 1, FALSE),
    ('pro', 'Pro Plan', 10, TRUE),
    ('enterprise', 'Enterprise Plan', 100, TRUE);

INSERT OR IGNORE INTO billing_profiles (tenant_id, has_active_payment_method, payment_provider_customer_id) VALUES
    ('tenant_no_payment', FALSE, NULL),
    ('tenant_with_payment', TRUE, 'cus_1234567890'),
    ('tenant_payment_expired', FALSE, 'cus_expired'),
    ('tenant_free_plan', FALSE, NULL);
