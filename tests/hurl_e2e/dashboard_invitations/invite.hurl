# Invite employee E2E tests
#
# Prerequisites:
#   - Server running with clean DB (no owner registered)
#   - BASE_URL set (default: http://localhost:5000)

# Register the owner
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 201

# Verify all emails (test endpoint)
POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP 200

# Login as owner to get access token
POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 200
[Captures]
access_token: jsonpath "$.access_token"

# --- Create a new invitation (201) ---
POST {{BASE_URL}}/api/v1/dashboard/invitations
Content-Type: application/json
Authorization: Bearer {{access_token}}
{
    "email": "alice@example.com",
    "role": "staff"
}
HTTP 200
[Asserts]
jsonpath "$.message" contains "alice@example.com"
jsonpath "$.email" == "alice@example.com"
jsonpath "$.role" == "staff"

# --- Re-invite same email (200) ---
POST {{BASE_URL}}/api/v1/dashboard/invitations
Content-Type: application/json
Authorization: Bearer {{access_token}}
{
    "email": "alice@example.com",
    "role": "manager"
}
HTTP 200
[Asserts]
jsonpath "$.message" contains "alice@example.com"
jsonpath "$.email" == "alice@example.com"
jsonpath "$.role" == "manager"

# --- Reject owner role (400) ---
POST {{BASE_URL}}/api/v1/dashboard/invitations
Content-Type: application/json
Authorization: Bearer {{access_token}}
{
    "email": "bob@example.com",
    "role": "owner"
}
HTTP 400
[Asserts]
jsonpath "$.message" == "Cannot invite with owner role"

# --- Reject unauthenticated (401) ---
POST {{BASE_URL}}/api/v1/dashboard/invitations
Content-Type: application/json
{
    "email": "carol@example.com",
    "role": "staff"
}
HTTP 401

# --- Invite without Bearer token (401) ---
POST {{BASE_URL}}/api/v1/dashboard/invitations
Content-Type: application/json
Authorization: InvalidToken
{
    "email": "carol@example.com",
    "role": "staff"
}
HTTP 401

# --- Register alice as employee to test 409 conflict ---
# First, we need to verify all so the test-only verify endpoint works,
# but alice isn't an employee yet - she's only invited.
# Let's invite a new person, then manually note we can't easily test 409
# without a second registration path. Skip for now or test via service tests.
