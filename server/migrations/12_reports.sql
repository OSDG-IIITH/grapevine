CREATE TABLE reports (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  course_id TEXT REFERENCES courses(id) ON DELETE CASCADE,
  faculty_id TEXT REFERENCES faculty(id) ON DELETE CASCADE,
  lab_id TEXT REFERENCES labs(id) ON DELETE CASCADE,
  offering_id TEXT REFERENCES offerings(id) ON DELETE CASCADE,
  reason TEXT NOT NULL CHECK (char_length(btrim(reason)) BETWEEN 3 AND 1000),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (num_nonnulls(course_id, faculty_id, lab_id, offering_id) = 1)
);

CREATE UNIQUE INDEX idx_reports_user_course ON reports(user_id, course_id) WHERE course_id IS NOT NULL;
CREATE UNIQUE INDEX idx_reports_user_faculty ON reports(user_id, faculty_id) WHERE faculty_id IS NOT NULL;
CREATE UNIQUE INDEX idx_reports_user_lab ON reports(user_id, lab_id) WHERE lab_id IS NOT NULL;
CREATE UNIQUE INDEX idx_reports_user_offering ON reports(user_id, offering_id) WHERE offering_id IS NOT NULL;
CREATE INDEX idx_reports_created_at ON reports(created_at DESC);
