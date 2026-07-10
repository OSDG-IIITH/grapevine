CREATE INDEX IF NOT EXISTS idx_course_reviews_offering_id ON course_reviews(offering_id);
CREATE INDEX IF NOT EXISTS idx_advisor_reviews_faculty_id ON advisor_reviews(faculty_id);
CREATE INDEX IF NOT EXISTS idx_offering_faculty_faculty_id ON offering_faculty(faculty_id);
