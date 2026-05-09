# 1. Register
POST {{BASE_URL}}/api/v1/auth/register
Content-Type: application/json
{
  "email": "test@example.com",
  "password": "password123"
}
HTTP/1.1 200

# 2. Auto-verify all users (test feature only)
POST {{BASE_URL}}/api/v1/auth/test/verify-all
HTTP/1.1 200

# 3. Login
POST {{BASE_URL}}/api/v1/auth/login
Content-Type: application/json
{
  "email": "test@example.com",
  "password": "password123"
}
HTTP/1.1 200
[Captures]
access_token: jsonpath "$.accessToken"
refresh_token: jsonpath "$.refreshToken"
[Asserts]
jsonpath "$.user.email" == "test@example.com"
jsonpath "$.user.role" == "customer"
jsonpath "$.accessToken" isString
jsonpath "$.refreshToken" isString

# 4. Get current user profile
GET {{BASE_URL}}/api/v1/auth/me
Authorization: Bearer {{access_token}}
HTTP/1.1 200
[Asserts]
jsonpath "$.email" == "test@example.com"
jsonpath "$.role" == "customer"
jsonpath "$.isActive" == true
jsonpath "$.emailVerifiedAt" exists
jsonpath "$.id" isInteger
jsonpath "$.createdAt" isString
jsonpath "$.updatedAt" isString
jsonpath "$.passwordHash" not exists
