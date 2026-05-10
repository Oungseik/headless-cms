# Setup: register, verify, login, refresh
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

POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
  "token": "{{refresh_token}}"
}
HTTP/1.1 200
[Captures]
new_access_token: jsonpath "$.accessToken"
new_refresh_token: jsonpath "$.refreshToken"

# Logout
POST {{BASE_URL}}/api/v1/dashboard/auth/logout
Content-Type: application/json
{
  "token": "{{new_refresh_token}}"
}
HTTP/1.1 200

# Refresh token no longer works after logout
POST {{BASE_URL}}/api/v1/dashboard/auth/refresh
Content-Type: application/json
{
  "token": "{{new_refresh_token}}"
}
HTTP/1.1 401

# Access token still works until it expires
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Bearer {{new_access_token}}
HTTP/1.1 200
[Asserts]
jsonpath "$.email" == "newbie@example.com"
