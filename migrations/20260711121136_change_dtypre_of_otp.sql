-- Add migration script here
ALTER TABLE student
ALTER COLUMN forgotten_pass_otp  TYPE TEXT USING forgotten_pass_otp::TEXT;