CREATE TABLE questions (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  course_id TEXT REFERENCES courses(id) ON DELETE CASCADE,
  faculty_id TEXT REFERENCES faculty(id) ON DELETE CASCADE,
  anonymous BOOLEAN NOT NULL DEFAULT false,
  body TEXT NOT NULL CHECK (char_length(btrim(body)) BETWEEN 10 AND 500),
  search TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', body)) STORED,
  edited_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  deleted_by TEXT REFERENCES users(id),
  CHECK (num_nonnulls(course_id, faculty_id) = 1)
);

CREATE TABLE answers (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  question_id TEXT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
  offering_id TEXT REFERENCES offerings(id) ON DELETE SET NULL,
  anonymous BOOLEAN NOT NULL DEFAULT false,
  body TEXT NOT NULL CHECK (char_length(btrim(body)) BETWEEN 2 AND 5000),
  search TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', body)) STORED,
  edited_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  deleted_by TEXT REFERENCES users(id)
);

CREATE TABLE question_votes (
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  question_id TEXT NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, question_id)
);

CREATE TABLE answer_votes (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  answer_id TEXT NOT NULL REFERENCES answers(id) ON DELETE CASCADE,
  vote SMALLINT NOT NULL CHECK (vote IN (1, -1)),
  UNIQUE (user_id, answer_id)
);

CREATE TABLE qa_flags (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  question_id TEXT REFERENCES questions(id) ON DELETE CASCADE,
  answer_id TEXT REFERENCES answers(id) ON DELETE CASCADE,
  reason TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (num_nonnulls(question_id, answer_id) = 1)
);

CREATE INDEX idx_questions_course ON questions(course_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_questions_faculty ON questions(faculty_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_questions_search ON questions USING GIN (search);
CREATE INDEX idx_answers_question ON answers(question_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_answers_search ON answers USING GIN (search);
CREATE UNIQUE INDEX idx_answers_unique ON answers(user_id, question_id) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_qa_flags_question ON qa_flags(user_id, question_id) WHERE question_id IS NOT NULL;
CREATE UNIQUE INDEX idx_qa_flags_answer ON qa_flags(user_id, answer_id) WHERE answer_id IS NOT NULL;
