# Login E2E tests
#
# Prerequisites:
#   - Server running with clean DB (no owner registered)
#   - BASE_URL set (default: http://localhost:5000)
#   - Recommend: RATE_LIMIT_ENABLED=false (login has rate limiter burst 3)

# --- Setup: register owner ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 201

# --- Login before verification returns 403 ---
POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 403

# --- Verify email ---
POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP 200

# --- Login with valid credentials (200) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 200
[Asserts]
jsonpath "$.access_token" isString
jsonpath "$.refresh_token" isString
jsonpath "$.token_type" == "Bearer"
jsonpath "$.expires_in" == 900
[Captures]
access_token: jsonpath "$.access_token"
refresh_token: jsonpath "$.refresh_token"

# --- Login with wrong password (401) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "wrongpassword"
}
HTTP 401

# --- Login with non-existent email (401) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
    "email": "ghost@example.com",
    "password": "password1234"
}
HTTP 401
