# Health check E2E tests

# --- Server responds with health message (200) ---
GET {{BASE_URL}}/health
HTTP 200
[Asserts]
body == "server is up and running"
