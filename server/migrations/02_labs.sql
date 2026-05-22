CREATE TABLE labs (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  shortname TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT
);
