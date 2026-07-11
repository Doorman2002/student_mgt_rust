
-- Add migration script here
ALTER TABLE student
ADD COLUMN email_verify TEXT DEFAULT 'nil',
ADD COLUMN email_otp_createdAT TIMESTAMPTZ;
 