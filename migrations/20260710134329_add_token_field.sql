-- Add migration script here
ALTER TABLE student
ADD COLUMN token_id TEXT DEFAULT '';
