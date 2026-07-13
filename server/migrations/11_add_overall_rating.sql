ALTER TABLE course_reviews ADD COLUMN overall REAL;
ALTER TABLE advisor_reviews ADD COLUMN overall REAL;

UPDATE course_reviews
SET overall = (difficulty + teaching + grading + content + workload)::real / 5.0;
UPDATE advisor_reviews
SET overall = (research + availability + mentorship + support + workload)::real / 5.0;

ALTER TABLE course_reviews ALTER COLUMN overall SET NOT NULL;
ALTER TABLE course_reviews ADD CONSTRAINT course_reviews_overall_check CHECK (overall BETWEEN 1.0 AND 5.0);
ALTER TABLE advisor_reviews ALTER COLUMN overall SET NOT NULL;
ALTER TABLE advisor_reviews ADD CONSTRAINT advisor_reviews_overall_check CHECK (overall BETWEEN 1.0 AND 5.0);
