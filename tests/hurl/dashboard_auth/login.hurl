# Setup: register and verify
POST {{BASE_URL}}/api/v1/dashboard/auth/register
Content-Type: application/json
{
  "email": "newbie@example.com",
  "password": "password123"
}
HTTP/1.1 201

POST {{BASE_URL}}/api/v1/dashboard/auth/test/verify-all
HTTP/1.1 200

# Login before email verification (should fail)
POST {{BASE_URL}}/api/v1/dashboard/auth/login
Content-Type: application/json
{
  "email": "newbie@example.com",
  "password": "password123"
}
HTTP/1.1 403
[Asserts]
jsonpath "$.message" == "Email not verified. Please check your email."

# Login after verification
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
[Asserts]
jsonpath "$.employee.email" == "newbie@example.com"
jsonpath "$.employee.role" == "owner"
jsonpath "$.employee.isActive" == true
jsonpath "$.accessToken" isString
jsonpath "$.refreshToken" isString

# Access /me without token
GET {{BASE_URL}}/api/v1/dashboard/auth/me
HTTP/1.1 401

# Access /me with garbage token
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Bearer not-a-real-token
HTTP/1.1 401

# Access /me with valid token
GET {{BASE_URL}}/api/v1/dashboard/auth/me
Authorization: Bearer {{access_token}}
HTTP/1.1 200
[Asserts]
jsonpath "$.email" == "newbie@example.com"
jsonpath "$.role" == "owner"
jsonpath "$.isActive" == true
jsonpath "$.emailVerifiedAt" exists
jsonpath "$.id" isString
jsonpath "$.createdAt" isString
jsonpath "$.updatedAt" isString
jsonpath "$.passwordHash" not exists
