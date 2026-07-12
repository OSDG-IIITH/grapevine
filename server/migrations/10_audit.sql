CREATE TABLE audit_logs (
    id TEXT PRIMARY KEY,
    admin_id TEXT NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    previous_state JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

ALTER TABLE course_reviews
    ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL,
    ADD COLUMN deleted_by TEXT REFERENCES users(id) DEFAULT NULL;

ALTER TABLE advisor_reviews
    ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL,
    ADD COLUMN deleted_by TEXT REFERENCES users(id) DEFAULT NULL;

ALTER TABLE offerings
    ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL,
    ADD COLUMN deleted_by TEXT REFERENCES users(id) DEFAULT NULL;

ALTER TABLE course_reviews DROP CONSTRAINT course_reviews_user_id_offering_id_key;
CREATE UNIQUE INDEX idx_course_reviews_unique ON course_reviews(user_id, offering_id) WHERE deleted_at IS NULL;

ALTER TABLE advisor_reviews DROP CONSTRAINT advisor_reviews_user_id_faculty_id_key;
CREATE UNIQUE INDEX idx_advisor_reviews_unique ON advisor_reviews(user_id, faculty_id) WHERE deleted_at IS NULL;
