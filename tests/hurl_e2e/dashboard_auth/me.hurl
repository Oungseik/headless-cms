# Get current employee profile E2E tests
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

# --- Get profile with valid token (200) ---
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.id" isString
jsonpath "$.email" == "owner@example.com"
jsonpath "$.role" == "owner"
jsonpath "$.is_active" == true
jsonpath "$.email_verified_at" isString
jsonpath "$.created_at" isString
jsonpath "$.updated_at" isString

# --- Missing Authorization header (401) ---
GET {{BASE_URL}}/api/v1/dashboard/auth/me
HTTP 401

# --- Invalid JWT token (401) ---
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Bearer invalid.jwt.token
HTTP 401

# --- Empty Bearer token (401) ---
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Bearer 
HTTP 401

# --- Wrong auth scheme (401) ---
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Basic abc123
HTTP 401
