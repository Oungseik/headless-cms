# Test rate limiting behavior
# With burst_size=10, the first 10 requests should succeed
# The 11th request should return 429

# Request 1 - should succeed
GET {{BASE_URL}}/health
HTTP/1.1 200
[Asserts]
header "X-RateLimit-Remaining" exists
header "X-RateLimit-Reset" exists

# Request 2
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 3
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 4
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 5
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 6
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 7
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 8
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 9
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 10 - should succeed (burst exhausted)
GET {{BASE_URL}}/health
HTTP/1.1 200

# Request 11 - should be rate limited
GET {{BASE_URL}}/health
HTTP/1.1 429
[Asserts]
header "Retry-After" exists
header "X-RateLimit-Remaining" exists
header "X-RateLimit-Reset" exists
jsonpath "$.message" == "Too Many Requests"
