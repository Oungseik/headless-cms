CREATE TABLE IF NOT EXISTS "employee" (
    "id" text NOT NULL PRIMARY KEY,
    "email" varchar NOT NULL UNIQUE,
    "password_hash" text NOT NULL DEFAULT '',
    "role" text NOT NULL DEFAULT 'owner',
    "is_active" boolean NOT NULL DEFAULT TRUE,
    "email_verified_at" timestamp_with_timezone_text,
    "created_at" timestamp_with_timezone_text NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp_with_timezone_text NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "employee_refresh_token" (
    "id" text NOT NULL PRIMARY KEY,
    "employee_id" text NOT NULL,
    "token_hash" varchar NOT NULL UNIQUE,
    "expires_at" timestamp_with_timezone_text NOT NULL,
    "revoked_at" timestamp_with_timezone_text,
    "created_at" timestamp_with_timezone_text NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY ("fk_employee_refresh_tokens_employee_id") REFERENCES "employee" ("id") ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS "employee_email_verification_token" (
    "id" text NOT NULL PRIMARY KEY,
    "employee_id" text NOT NULL,
    "token_hash" varchar NOT NULL UNIQUE,
    "expires_at" timestamp_with_timezone_text NOT NULL,
    "created_at" timestamp_with_timezone_text NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (
        "fk_employee_email_verification_tokens_employee_id"
    ) REFERENCES "employee" ("id") ON DELETE CASCADE
);

CREATE INDEX "idx_employee_email_verification_tokens_employee_id" ON "employee_email_verification_token" ("employee_id");
