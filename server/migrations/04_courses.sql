CREATE TABLE courses (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  code TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT,
  deleted_at TIMESTAMPTZ DEFAULT NULL,
  shortnames TEXT[] NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_courses_name_trgm ON courses USING GIN (name gin_trgm_ops);
CREATE INDEX idx_courses_code_trgm ON courses USING GIN (code gin_trgm_ops);

CREATE TABLE course_succession (
    predecessor_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    successor_id   TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    PRIMARY KEY (predecessor_id, successor_id),
    CHECK (predecessor_id <> successor_id)
);

