-- Add migration script here


ALTER TABLE student
ALTER COLUMN forgotten_pass_otp  TYPE UUID USING forgotten_pass_otp::UUID;
