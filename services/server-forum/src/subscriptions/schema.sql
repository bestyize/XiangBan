CREATE TABLE IF NOT EXISTS information_subscriptions (
 id UUID PRIMARY KEY, owner UUID NOT NULL REFERENCES users(uid), revision BIGINT NOT NULL DEFAULT 1,
 title TEXT NOT NULL, interests TEXT NOT NULL, recipient TEXT NOT NULL,
 schedule JSONB NOT NULL, timezone TEXT NOT NULL, next_run TIMESTAMPTZ,
 status TEXT NOT NULL CHECK(status IN ('unverified','active','paused','quota','completed','deleted')),
 verified_at TIMESTAMPTZ, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS information_due ON information_subscriptions(next_run) WHERE status='active';
CREATE TABLE IF NOT EXISTS information_verifications (
 subscription_id UUID PRIMARY KEY REFERENCES information_subscriptions(id), digest BYTEA NOT NULL,
 expires_at TIMESTAMPTZ NOT NULL, attempts INT NOT NULL DEFAULT 0, sent_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS information_editions (
 id UUID PRIMARY KEY, subscription_id UUID NOT NULL REFERENCES information_subscriptions(id), owner UUID NOT NULL,
 revision BIGINT NOT NULL, due_at TIMESTAMPTZ NOT NULL, snapshot JSONB NOT NULL,
 status TEXT NOT NULL DEFAULT 'queued', attempts INT NOT NULL DEFAULT 0, available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 lease_until TIMESTAMPTZ, lease_id UUID, html TEXT, plaintext TEXT, sources JSONB, provider_id TEXT,
 error TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), sent_at TIMESTAMPTZ,
 UNIQUE(subscription_id, revision, due_at)
);
CREATE INDEX IF NOT EXISTS information_edition_owner ON information_editions(owner, created_at DESC);
CREATE TABLE IF NOT EXISTS information_usage (
 edition_id UUID NOT NULL REFERENCES information_editions(id), attempt INT NOT NULL, stage TEXT NOT NULL,
 usage JSONB NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(edition_id, attempt, stage)
);
-- Counts requests even when delivery fails; cannot be reset by deleting a subscription.
CREATE TABLE IF NOT EXISTS information_limits (
 owner UUID NOT NULL, day DATE NOT NULL, creates INT NOT NULL DEFAULT 0, verifications INT NOT NULL DEFAULT 0,
 PRIMARY KEY(owner,day)
);
