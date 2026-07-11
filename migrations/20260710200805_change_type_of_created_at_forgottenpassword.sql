-- Add migration script here
ALTER TABLE student
ALTER COLUMN forgotten_pass_createdAt TYPE TIMESTAMPTZ USING forgotten_pass_createdAt::TIMESTAMPTZ;