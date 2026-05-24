CREATE TABLE faculty (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  bio TEXT,
  lab_id TEXT REFERENCES labs(id) ON DELETE SET NULL
);

CREATE INDEX idx_faculty_name_trgm ON faculty USING GIN (name gin_trgm_ops);
