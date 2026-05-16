# Logout E2E tests
#
# Prerequisites:
#   - Server running with clean DB (no owner registered)
#   - BASE_URL set (default: http://localhost:5000)

# --- Setup: register, verify, login ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 201

POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP 200

POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 200
[Captures]
refresh_token: jsonpath "$.refresh_token"

# --- Logout with valid refresh token (200) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/logout
Content-Type: application/json
{
    "token": "{{refresh_token}}"
}
HTTP 200
[Asserts]
jsonpath "$.message" == "Logged out successfully."

# --- Logout with same (revoked) token still succeeds (idempotent) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/logout
Content-Type: application/json
{
    "token": "{{refresh_token}}"
}
HTTP 200
[Asserts]
jsonpath "$.message" == "Logged out successfully."

# --- Logout with invalid hex token still succeeds ---
POST {{BASE_URL}}/api/v1/dashboard/auth/logout
Content-Type: application/json
{
    "token": "not-valid-hex"
}
HTTP 200
[Asserts]
jsonpath "$.message" == "Logged out successfully."
