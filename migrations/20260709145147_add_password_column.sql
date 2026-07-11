-- Add migration script here
ALTER TABLE student
ADD COLUMN password TEXT NOT NULL;