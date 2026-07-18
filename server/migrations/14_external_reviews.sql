CREATE TABLE external_course_reviews (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  offering_id TEXT REFERENCES offerings(id) ON DELETE SET NULL,
  body TEXT NOT NULL,
  source_note TEXT,
  added_by TEXT NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE external_advisor_reviews (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  faculty_id TEXT NOT NULL REFERENCES faculty(id) ON DELETE CASCADE,
  body TEXT NOT NULL,
  source_note TEXT,
  added_by TEXT NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE external_course_review_votes (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  review_id TEXT NOT NULL REFERENCES external_course_reviews(id) ON DELETE CASCADE,
  vote SMALLINT NOT NULL CHECK (vote IN (1, -1)),
  UNIQUE (user_id, review_id)
);

CREATE TABLE external_advisor_review_votes (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  review_id TEXT NOT NULL REFERENCES external_advisor_reviews(id) ON DELETE CASCADE,
  vote SMALLINT NOT NULL CHECK (vote IN (1, -1)),
  UNIQUE (user_id, review_id)
);

CREATE INDEX idx_external_course_reviews_course ON external_course_reviews(course_id);
CREATE INDEX idx_external_advisor_reviews_faculty ON external_advisor_reviews(faculty_id);
