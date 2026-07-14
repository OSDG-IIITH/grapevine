CREATE TABLE legacy_course_reviews (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  body TEXT,
  course_id TEXT REFERENCES courses(id) ON DELETE CASCADE,
  offering_id TEXT REFERENCES offerings(id) ON DELETE SET NULL,
  original_rating SMALLINT,
  score BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_legacy_course_reviews_course ON legacy_course_reviews(course_id);

CREATE TABLE legacy_advisor_reviews (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  body TEXT,
  faculty_id TEXT REFERENCES faculty(id) ON DELETE CASCADE,
  original_rating SMALLINT,
  score BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_legacy_advisor_reviews_faculty ON legacy_advisor_reviews(faculty_id);

CREATE TABLE legacy_course_review_votes (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  review_id TEXT NOT NULL REFERENCES legacy_course_reviews(id) ON DELETE CASCADE,
  vote SMALLINT NOT NULL CHECK (vote IN (1, -1)),
  UNIQUE (user_id, review_id)
);

CREATE TABLE legacy_advisor_review_votes (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  review_id TEXT NOT NULL REFERENCES legacy_advisor_reviews(id) ON DELETE CASCADE,
  vote SMALLINT NOT NULL CHECK (vote IN (1, -1)),
  UNIQUE (user_id, review_id)
);
