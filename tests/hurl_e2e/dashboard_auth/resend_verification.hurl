# Resend verification email E2E tests
#
# Prerequisites:
#   - Server running with clean DB (no owner registered)
#   - BASE_URL set (default: http://localhost:5000)

# --- Register owner (unverified) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 201

# --- Resend verification for unverified account (200) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/email/verification/resend
Content-Type: application/json
{
    "email": "owner@example.com"
}
HTTP 200
[Asserts]
jsonpath "$.message" == "Verification email sent"

# --- Resend for non-existent email (404) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/email/verification/resend
Content-Type: application/json
{
    "email": "nobody@example.com"
}
HTTP 404

# --- Verify email via test endpoint ---
POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP 200

# --- Resend for already-verified account (409) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/email/verification/resend
Content-Type: application/json
{
    "email": "owner@example.com"
}
HTTP 409
[Asserts]
jsonpath "$.message" == "Email already verified"
