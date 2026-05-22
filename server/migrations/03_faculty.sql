CREATE TABLE faculty (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  bio TEXT,
  lab_id TEXT REFERENCES labs(id) ON DELETE SET NULL
);
