# Token refresh E2E tests
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
access_token: jsonpath "$.access_token"
refresh_token: jsonpath "$.refresh_token"

# --- Refresh with valid token returns new tokens (200) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
    "token": "{{refresh_token}}"
}
HTTP 200
[Asserts]
jsonpath "$.access_token" isString
jsonpath "$.refresh_token" isString
jsonpath "$.token_type" == "Bearer"
jsonpath "$.expires_in" == 900
[Captures]
new_access_token: jsonpath "$.access_token"
new_refresh_token: jsonpath "$.refresh_token"

# --- Old refresh token is revoked after rotation (401) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
    "token": "{{refresh_token}}"
}
HTTP 401

# --- New refresh token works (200) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
    "token": "{{new_refresh_token}}"
}
HTTP 200
[Captures]
rotated_access_token: jsonpath "$.access_token"
rotated_refresh_token: jsonpath "$.refresh_token"

# --- Refresh with completely invalid token (401) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
    "token": "deadbeef"
}
HTTP 401

# --- Logout the latest token, then try refresh (401) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/logout
Content-Type: application/json
{
    "token": "{{rotated_refresh_token}}"
}
HTTP 200

POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
    "token": "{{rotated_refresh_token}}"
}
HTTP 401
