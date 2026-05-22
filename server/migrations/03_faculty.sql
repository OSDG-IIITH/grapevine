CREATE TABLE faculty (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  bio TEXT,
  lab_id UUID REFERENCES labs(id) ON DELETE SET NULL
);
