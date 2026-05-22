CREATE TABLE course_reviews (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  offering_id TEXT NOT NULL REFERENCES offerings(id) ON DELETE CASCADE,
  anonymous BOOLEAN NOT NULL DEFAULT false,
  difficulty SMALLINT NOT NULL CHECK (difficulty BETWEEN 1 AND 5),
  teaching SMALLINT NOT NULL CHECK (teaching BETWEEN 1 AND 5),
  grading SMALLINT NOT NULL CHECK (grading BETWEEN 1 AND 5),
  content SMALLINT NOT NULL CHECK (content BETWEEN 1 AND 5),
  workload SMALLINT NOT NULL CHECK (workload BETWEEN 1 AND 5),
  body TEXT NOT NULL,
  edited_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, offering_id)
);

CREATE TABLE advisor_reviews (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  faculty_id TEXT NOT NULL REFERENCES faculty(id) ON DELETE CASCADE,
  anonymous BOOLEAN NOT NULL DEFAULT false,
  research SMALLINT NOT NULL CHECK (research BETWEEN 1 AND 5),
  availability SMALLINT NOT NULL CHECK (availability BETWEEN 1 AND 5),
  mentorship SMALLINT NOT NULL CHECK (mentorship BETWEEN 1 AND 5),
  support SMALLINT NOT NULL CHECK (support BETWEEN 1 AND 5),
  workload SMALLINT NOT NULL CHECK (workload BETWEEN 1 AND 5),
  body TEXT NOT NULL,
  edited_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, faculty_id)
);
