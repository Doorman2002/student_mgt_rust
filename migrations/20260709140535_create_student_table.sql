-- Add migration script here
CREATE TABLE student(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    phone TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    course TEXT,
    gurantor TEXT,
    paymentAmount INT,
    paymentDate DATE,
    paymentStatus TEXT,
    createdAT TIMESTAMPTZ DEFAULT NOW()
    
);