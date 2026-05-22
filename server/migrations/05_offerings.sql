CREATE TYPE offering_season AS ENUM ('M', 'S');

CREATE TABLE offerings (
  id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
  course_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  season offering_season NOT NULL,
  year SMALLINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (course_id, season, year)
);

CREATE TABLE offering_faculty (
  offering_id TEXT NOT NULL REFERENCES offerings(id) ON DELETE CASCADE,
  faculty_id TEXT NOT NULL REFERENCES faculty(id) ON DELETE CASCADE,
  PRIMARY KEY (offering_id, faculty_id)
);
