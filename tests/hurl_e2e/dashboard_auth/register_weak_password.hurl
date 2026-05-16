# Registration — weak password E2E test
#
# Must run on a clean DB (no owner registered yet).
#
# Prerequisites:
#   - Server running with clean DB
#   - BASE_URL set (default: http://localhost:5000)

# --- Weak password returns 400 ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "short"
}
HTTP 400
[Asserts]
jsonpath "$.message" == "Password must be at least 8 characters"

# --- Exactly 8 characters should succeed ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "12345678"
}
HTTP 201
[Asserts]
jsonpath "$.message" == "Please check your email to verify your account."
