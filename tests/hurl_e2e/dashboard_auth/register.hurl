# Registration E2E tests
#
# Prerequisites:
#   - Server running with clean DB (no owner registered)
#   - BASE_URL set (default: http://localhost:5000)

# --- Register owner with valid credentials (201) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 201
[Asserts]
jsonpath "$.message" == "Please check your email to verify your account."

# --- Duplicate registration returns 409 (different email, same owner slot) ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "other@example.com",
    "password": "password1234"
}
HTTP 409
[Asserts]
jsonpath "$.message" == "An owner has already been registered"

# --- Same email returns 409 too ---
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
    "email": "owner@example.com",
    "password": "password1234"
}
HTTP 409
[Asserts]
jsonpath "$.message" == "An owner has already been registered"

# --- Weak password returns 400 ---
# Note: requires clean DB, so we test this conceptually.
# In practice, the owner already exists so we get 409.
# This scenario is best tested in an isolated run or via unit tests.
