-- Create employee table
CREATE TABLE IF NOT EXISTS employee (
    id TEXT NOT NULL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL DEFAULT '',
    role TEXT NOT NULL DEFAULT 'owner',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    email_verified_at TEXT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create employee_refresh_token table
CREATE TABLE IF NOT EXISTS employee_refresh_token (
    id TEXT NOT NULL PRIMARY KEY,
    employee_id TEXT NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    revoked_at TEXT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);

-- Create employee_email_verification_token table
CREATE TABLE IF NOT EXISTS employee_email_verification_token (
    id TEXT NOT NULL PRIMARY KEY,
    employee_id TEXT NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (employee_id) REFERENCES employee(id) ON DELETE CASCADE
);

-- Create index on employee_email_verification_token.employee_id
CREATE INDEX IF NOT EXISTS idx_employee_email_verification_tokens_employee_id
    ON employee_email_verification_token(employee_id);
