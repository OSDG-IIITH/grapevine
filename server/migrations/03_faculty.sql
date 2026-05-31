CREATE TABLE faculty (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  bio TEXT
);

CREATE TABLE faculty_labs (
  faculty_id TEXT NOT NULL REFERENCES faculty(id) ON DELETE CASCADE,
  lab_id TEXT NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  PRIMARY KEY (faculty_id, lab_id)
);

CREATE INDEX idx_faculty_name_trgm ON faculty USING GIN (name gin_trgm_ops);
