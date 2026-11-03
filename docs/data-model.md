users
  id, cas_id (nullable, unique), username (nullable, unique), password_hash (nullable), display_name, is_admin (bool), verified (bool), created_at
  -- constraints:
  --   Must have either cas_id or username
  --   If username is present, password_hash must be present
  --   cas_id is unique (for non-null values)

verified_emails
  email_hash (text, PK)
  -- Stores SHA-256 HMACs of verified student emails to prevent duplicate local accounts while preserving anonymity.

courses
  id, code (unique), name, description, type (core/open/breadth/stream/bouquet/hs/sci/math)

offerings
  id, course_id, season (M/S), year (YY), created_at
  unique (course_id, season, year)

offering_faculty
  offering_id, faculty_id
  -- one offering can have multiple faculty

faculty
  id, slug (unique), name, bio

faculty_labs
  faculty_id, lab_id
  -- many-to-many: one faculty member can belong to multiple labs

labs
  id, shortname (unique), name, description

course_reviews
  id, user_id, offering_id
  anonymous (bool)
  difficulty, teaching, grading, content, workload (1–5 each)
  body (text)
  edited_at, created_at
  unique (user_id, offering_id)

advisor_reviews
  id, user_id, faculty_id
  anonymous (bool)
  research, availability, mentorship, support, workload (1–5 each)
  body (text)
  edited_at, created_at
  unique (user_id, faculty_id)

course_review_votes
  id, user_id, review_id (→ course_reviews), vote (+1/-1)
  unique (user_id, review_id)

advisor_review_votes
  id, user_id, review_id (→ advisor_reviews), vote (+1/-1)
  unique (user_id, review_id)

course_review_flags
  id, user_id, review_id (→ course_reviews), reason, created_at
  unique (user_id, review_id)

advisor_review_flags
  id, user_id, review_id (→ advisor_reviews), reason, created_at
  unique (user_id, review_id)

questions
  id, user_id, course_id (nullable), faculty_id (nullable)
  anonymous (bool)
  body (text, 10–500 chars), search (generated tsvector)
  edited_at, created_at, deleted_at, deleted_by
  -- exactly one of course_id, faculty_id is set

answers
  id, user_id, question_id, offering_id (nullable, course questions only)
  anonymous (bool)
  body (text, 2–5000 chars), search (generated tsvector)
  edited_at, created_at, deleted_at, deleted_by
  unique (user_id, question_id) where deleted_at is null

question_votes
  user_id, question_id ("me too", no value)
  primary key (user_id, question_id)

answer_votes
  id, user_id, answer_id, vote (+1/-1)
  unique (user_id, answer_id)

qa_flags
  id, user_id, question_id (nullable), answer_id (nullable), reason, created_at
  -- exactly one of question_id, answer_id is set; one flag per user per target
