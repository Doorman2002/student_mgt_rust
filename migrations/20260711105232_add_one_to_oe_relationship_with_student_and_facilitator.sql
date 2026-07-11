-- Add migration script here
CREATE TABLE tutor(
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
name TEXT ,
course TEXT


);
ALTER TABLE student
ADD COLUMN tutor_id UUID REFERENCES tutor(id);

