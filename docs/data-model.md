users
  id, cas_id, display_name, created_at

courses
  id, code, name, description
  type (core/open/breadth/stream/bouquet/hs/sci/math)

offerings
  id, course_id, season (M/S), year (YY), created_at

offering_faculty
  offering_id, faculty_id
  -- one offering can have multiple faculty

faculty
  id, name, bio, lab_id (nullable → labs)

labs
  id, name, description

course_reviews
  id, user_id, offering_id
  anonymous (bool)
  difficulty, teaching, grading, content, workload (1–5 each)
  body (text)
  edited_at, created_at

advisor_reviews
  id, user_id, faculty_id
  anonymous (bool)
  research, availability, mentorship, support, workload (1–5 each)
  body (text)
  edited_at, created_at

course_review_votes
  id, user_id, review_id (→ course_reviews), vote (+1/-1)
  unique (user_id, review_id)

advisor_review_votes
  id, user_id, review_id (→ advisor_reviews), vote (+1/-1)
  unique (user_id, review_id)

course_review_flags
  id, user_id, review_id (→ course_reviews), reason, created_at

advisor_review_flags
  id, user_id, review_id (→ advisor_reviews), reason, created_at
