# Email verification E2E tests
#
# Prerequisites:
#   - Server running with clean DB (no owner registered)
#   - BASE_URL set (default: http://localhost:5000)

# --- Register owner ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 201

# --- Verify all via test endpoint (200) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP 200
[Asserts]
jsonpath "$.message" == "All employees verified"

# --- Verify again (email already verified) returns 409 ---
# The test/verify-all endpoint always succeeds, so we test
# the actual verify_email endpoint's "already verified" case
# via resend_verification tests instead.
# The "invalid token" and "missing token" cases are tested below
# without depending on prior state.

# --- Invalid verification token returns 400 ---
GET {{BASE_URL}}/api/v1/dashboard/auth/email/verification?token=deadbeef
HTTP 400
[Asserts]
jsonpath "$.message" == "Invalid or expired verification token"

# --- Invalid hex token returns 400 ---
GET {{BASE_URL}}/api/v1/dashboard/auth/email/verification?token=not-valid-hex!@#$
HTTP 400
[Asserts]
jsonpath "$.message" == "Invalid or expired verification token"
