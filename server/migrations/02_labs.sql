CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE labs (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  shortname TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT
);

CREATE INDEX idx_labs_name_trgm ON labs USING GIN (name gin_trgm_ops);
CREATE INDEX idx_labs_shortname_trgm ON labs USING GIN (shortname gin_trgm_ops);
