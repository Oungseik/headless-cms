# Register with weak password (should fail)
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
  "email": "newbie@example.com",
  "password": "pass"
}
HTTP/1.1 400
[Asserts]
jsonpath "$.message" == "Password must be at least 8 characters"

# Register with proper password
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
  "email": "newbie@example.com",
  "password": "password123"
}
HTTP/1.1 201
[Asserts]
jsonpath "$.message" == "Please check your email to verify your account."

# Verify email (simulated via test endpoint)
POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP/1.1 200
[Asserts]
jsonpath "$.message" == "All employees verified"
