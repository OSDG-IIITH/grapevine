CREATE TABLE review_drafts (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  course_id TEXT REFERENCES courses(id) ON DELETE CASCADE,
  faculty_id TEXT REFERENCES faculty(id) ON DELETE CASCADE,
  data JSONB NOT NULL DEFAULT '{}',
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK ((course_id IS NULL) <> (faculty_id IS NULL)),
  UNIQUE (user_id, course_id),
  UNIQUE (user_id, faculty_id)
);
