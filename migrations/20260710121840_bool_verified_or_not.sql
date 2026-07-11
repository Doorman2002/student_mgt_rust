-- Add migration script here
ALTER TABLE student
ADD COLUMN verified BOOLEAN DEFAULT FALSE;