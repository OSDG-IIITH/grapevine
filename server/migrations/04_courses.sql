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
