-- Create invitation table
CREATE TABLE IF NOT EXISTS invitation (
    id TEXT NOT NULL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    role TEXT NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    invited_by TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    accepted_at TEXT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (invited_by) REFERENCES employee(id) ON DELETE CASCADE
);

-- Create index on invitation.email for pending-invitation lookups
CREATE INDEX IF NOT EXISTS idx_invitation_email: email_sender ON invitation(email);

-- Ensure at most one pending invitation per email
CREATE UNIQUE INDEX IF NOT EXISTS idx_invitation_pending_email ON invitation(email)
WHERE
    accepted_at IS NULL;
