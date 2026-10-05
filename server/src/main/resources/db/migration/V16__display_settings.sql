create table site_display_settings (
    id smallint primary key check (id = 1),
    config jsonb not null check (jsonb_typeof(config) = 'object'),
    updated_at timestamptz not null default now()
);
