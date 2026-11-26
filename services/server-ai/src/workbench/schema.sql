CREATE TABLE IF NOT EXISTS workbench_sessions (
    id UUID PRIMARY KEY,
    uid UUID NOT NULL,
    product_id TEXT NOT NULL REFERENCES products(product_id),
    title TEXT NOT NULL DEFAULT '新对话',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS workbench_sessions_owner ON workbench_sessions(uid, product_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS workbench_runs (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES workbench_sessions(id) ON DELETE CASCADE,
    request_id UUID NOT NULL,
    input TEXT NOT NULL,
    tier TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running','done','cancelled','error')),
    heartbeat TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    history JSONB NOT NULL DEFAULT '[]',
    UNIQUE (session_id, request_id)
);
CREATE UNIQUE INDEX IF NOT EXISTS workbench_active ON workbench_runs(session_id) WHERE status='running';
CREATE INDEX IF NOT EXISTS workbench_stale ON workbench_runs(heartbeat) WHERE status='running';

CREATE TABLE IF NOT EXISTS workbench_events (
    seq BIGSERIAL PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES workbench_sessions(id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES workbench_runs(id) ON DELETE CASCADE,
    payload JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS workbench_replay ON workbench_events(session_id,seq);

CREATE TABLE IF NOT EXISTS workbench_approvals (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES workbench_runs(id) ON DELETE CASCADE,
    decision TEXT CHECK (decision IN ('once','deny'))
);
