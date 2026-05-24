CREATE TYPE course_type AS ENUM (
  'core', 'open', 'breadth', 'stream', 'bouquet', 'hs', 'sci', 'math'
);

CREATE TABLE courses (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  code TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT,
  type course_type NOT NULL
);

CREATE INDEX idx_courses_name_trgm ON courses USING GIN (name gin_trgm_ops);
CREATE INDEX idx_courses_code_trgm ON courses USING GIN (code gin_trgm_ops);
