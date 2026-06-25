-- Drop the timestamp from the used-email set: it served no functional purpose
-- (the email_hash PK handles dedup) and let a DB admin correlate a verified_emails
-- row to the user who verified at that instant, weakening the anonymity guarantee.
ALTER TABLE verified_emails DROP COLUMN IF EXISTS created_at;
