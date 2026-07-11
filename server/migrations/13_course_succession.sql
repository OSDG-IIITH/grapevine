CREATE TABLE course_succession (
    predecessor_id TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    successor_id   TEXT NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    PRIMARY KEY (predecessor_id, successor_id),
    CHECK (predecessor_id <> successor_id)
);
