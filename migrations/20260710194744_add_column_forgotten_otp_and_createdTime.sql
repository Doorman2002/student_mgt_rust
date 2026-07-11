-- Add migration script here
ALTER TABLE student
ADD COLUMN forgotten_pass_otp TEXT NULL,
ADD COLUMN forgotten_pass_createdAt TEXT NULL;