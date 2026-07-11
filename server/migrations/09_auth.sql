-- Anonymous username/password auth: extend users + used-email set.

-- (1) add columns
ALTER TABLE users ALTER COLUMN cas_id DROP NOT NULL;
ALTER TABLE users ADD COLUMN username TEXT UNIQUE;
ALTER TABLE users ADD COLUMN password_hash TEXT;
ALTER TABLE users ADD COLUMN verified BOOLEAN NOT NULL DEFAULT false;

-- (2) backfill: existing CAS rows are already verified
UPDATE users SET verified = true WHERE cas_id IS NOT NULL;

-- (3) constraints: a row is either a CAS row or a local row; local rows need a password
ALTER TABLE users ADD CONSTRAINT users_cas_or_username
  CHECK (cas_id IS NOT NULL OR username IS NOT NULL);
ALTER TABLE users ADD CONSTRAINT users_username_requires_password
  CHECK (username IS NULL OR password_hash IS NOT NULL);

-- used-email set: HMAC hashes only, no plaintext, no link to any account
CREATE TABLE verified_emails (
  email_hash TEXT PRIMARY KEY
);

-- recovery methods for local accounts
ALTER TABLE users ADD COLUMN recovery_code_hash TEXT;
ALTER TABLE users ADD COLUMN security_question TEXT;
ALTER TABLE users ADD COLUMN security_answer_hash TEXT;
