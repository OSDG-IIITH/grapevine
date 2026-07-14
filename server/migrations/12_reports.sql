CREATE TABLE reports (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  course_id TEXT REFERENCES courses(id) ON DELETE CASCADE,
  faculty_id TEXT REFERENCES faculty(id) ON DELETE CASCADE,
  lab_id TEXT REFERENCES labs(id) ON DELETE CASCADE,
  offering_id TEXT REFERENCES offerings(id) ON DELETE CASCADE,
  has_faculty_suggestion BOOLEAN NOT NULL DEFAULT false,
  reason TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (num_nonnulls(course_id, faculty_id, lab_id, offering_id) = 1),
  CHECK (NOT has_faculty_suggestion OR offering_id IS NOT NULL),
  CHECK (
    (offering_id IS NOT NULL AND (char_length(btrim(reason)) = 0 OR char_length(btrim(reason)) BETWEEN 3 AND 1000))
    OR (offering_id IS NULL AND char_length(btrim(reason)) BETWEEN 3 AND 1000)
  )
);

CREATE UNIQUE INDEX idx_reports_user_course ON reports(user_id, course_id) WHERE course_id IS NOT NULL;
CREATE UNIQUE INDEX idx_reports_user_faculty ON reports(user_id, faculty_id) WHERE faculty_id IS NOT NULL;
CREATE UNIQUE INDEX idx_reports_user_lab ON reports(user_id, lab_id) WHERE lab_id IS NOT NULL;
CREATE UNIQUE INDEX idx_reports_user_offering ON reports(user_id, offering_id) WHERE offering_id IS NOT NULL;
CREATE INDEX idx_reports_created_at ON reports(created_at DESC);

CREATE TABLE report_faculty (
  report_id TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
  faculty_id TEXT NOT NULL REFERENCES faculty(id),
  suggested BOOLEAN NOT NULL,
  PRIMARY KEY (report_id, faculty_id, suggested)
);
