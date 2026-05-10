# Setup: register, verify, login
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
  "email": "newbie@example.com",
  "password": "password123"
}
HTTP/1.1 201

POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP/1.1 200

POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
  "email": "newbie@example.com",
  "password": "password123"
}
HTTP/1.1 200
[Captures]
access_token: jsonpath "$.accessToken"
refresh_token: jsonpath "$.refreshToken"

# Refresh token
POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
  "token": "{{refresh_token}}"
}
HTTP/1.1 200
[Captures]
new_access_token: jsonpath "$.accessToken"
new_refresh_token: jsonpath "$.refreshToken"
[Asserts]
jsonpath "$.accessToken" isString
jsonpath "$.refreshToken" isString

# Verify new token works
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Bearer {{new_access_token}}
HTTP/1.1 200
[Asserts]
jsonpath "$.email" == "newbie@example.com"
