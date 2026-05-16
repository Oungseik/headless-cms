# API Test Plan

**Generated**: 2026-05-16
**API Version**: v1
**Base URL**: `{{BASE_URL}}` (default: `http://localhost:5000`)

## Overview

Headless CMS dashboard API with authentication (register/verify/login/logout/refresh), profile retrieval, and invitation management. Only one owner account can be registered; additional employees join via invitations.

## Endpoints Summary

| Method | Path | Auth | Tag |
|--------|------|------|-----|
| GET | `/health` | No | Health |
| POST | `/api/v1/dashboard/auth/register` | No | Dashboard Auth |
| GET | `/api/v1/dashboard/auth/email/verification?token={token}` | No | Dashboard Auth |
| POST | `/api/v1/dashboard/auth/email/verification/resend` | No | Dashboard Auth |
| POST | `/api/v1/dashboard/auth/login` | No | Dashboard Auth |
| POST | `/api/v1/dashboard/auth/logout` | No | Dashboard Auth |
| POST | `/api/v1/dashboard/auth/refresh` | No | Dashboard Auth |
| GET | `/api/v1/dashboard/auth/me` | Bearer JWT | Dashboard Auth |
| POST | `/api/v1/dashboard/invitations` | Bearer JWT (owner) | Dashboard Invitations |
| POST | `/api/v1/dashboard/auth/test/verify-all` | No | Dashboard Auth (Testing only) |

## Dependencies

```
Register → Verify Email → Login → { Me, Logout, Refresh, Invite }
                    ↘ Resend Verification ↗
Login → Refresh → Me (token rotation)
Login → Logout → Refresh (should fail)
```

---

## Flow 1: Health Check

**Endpoints**: `GET /health`

### 1.1 Happy path — server responds
- **Priority**: P0
- **Steps**: `GET /health`
- **Expected**: 200, body `"server is up and running"`

---

## Flow 2: Owner Registration

**Endpoints**:
- `POST /api/v1/dashboard/auth/register`
- `GET /api/v1/dashboard/auth/email/verification?token={token}`
- `POST /api/v1/dashboard/auth/email/verification/resend`

### 2.1 Happy path — register owner, verify, login (P0)
1. `POST /register` with `{ "email": "owner@example.com", "password": "password1234" }`
2. Assert 201, body contains `"message": "Please check your email to verify your account."`
3. `POST /test/verify-all` → 200 (shortcut for E2E; real flow uses email token)
4. `POST /login` → 200, capture `access_token`, `refresh_token`

### 2.2 Register — weak password (P0)
1. `POST /register` with `{ "email": "owner@example.com", "password": "short" }`
2. Assert 400, `$.message == "Password must be at least 8 characters"`

### 2.3 Register — duplicate owner (P0)
1. Register first owner (201)
2. Register again with different email → 409
3. Assert `$.message == "An owner has already been registered"`

### 2.4 Register — missing fields (P1)
1. `POST /register` with `{}` → 400 (Axum deserialization error)
2. `POST /register` with `{ "email": "" }` → 400

### 2.5 Register — invalid email format (P2)
1. `POST /register` with `{ "email": "not-an-email", "password": "password1234" }`
2. Note: server may accept any string (no email format validation in code). Verify behavior.

### 2.6 Verify email — valid token (P0)
1. Register → capture token hex from service layer or DB
2. `GET /email/verification?token={hex}` → 200
3. `$.message == "Email verified successfully."`

### 2.7 Verify email — invalid/expired token (P1)
1. `GET /email/verification?token=deadbeef` → 400
2. `$.message == "Invalid or expired verification token"`

### 2.8 Verify email — already verified (P1)
1. Register → verify → verify again with same token → 409
2. `$.message == "Email already verified"`

### 2.9 Verify email — missing token param (P1)
1. `GET /email/verification` (no query param) → 400

### 2.10 Resend verification — happy path (P1)
1. Register (don't verify) → `POST /email/verification/resend` with `{ "email": "owner@example.com" }`
2. Assert 200, `$.message == "Verification email sent"`

### 2.11 Resend verification — account not found (P1)
1. `POST /email/verification/resend` with `{ "email": "nobody@example.com" }` → 404

### 2.12 Resend verification — already verified (P1)
1. Register → verify → `POST /email/verification/resend` → 409
2. `$.message == "Email already verified"`

---

## Flow 3: Authentication (Login / Logout / Refresh)

**Endpoints**:
- `POST /api/v1/dashboard/auth/login`
- `POST /api/v1/dashboard/auth/logout`
- `POST /api/v1/dashboard/auth/refresh`
- `GET /api/v1/dashboard/auth/me`

### 3.1 Login — valid credentials (P0)
1. Register → verify → `POST /login` with correct email/password
2. Assert 200, `$.access_token` is non-empty string, `$.refresh_token` is hex, `$.token_type == "Bearer"`, `$.expires_in == 900`

### 3.2 Login — wrong password (P0)
1. `POST /login` with wrong password → 401

### 3.3 Login — non-existent email (P0)
1. `POST /login` with `{ "email": "ghost@example.com", "password": "password1234" }` → 401

### 3.4 Login — unverified email (P0)
1. Register (don't verify) → `POST /login` → 403

### 3.5 Login — missing fields (P1)
1. `POST /login` with `{}` → 400 (Axum deserialization)

### 3.6 Logout — valid refresh token (P0)
1. Login → capture `refresh_token`
2. `POST /logout` with `{ "token": "{refresh_token}" }` → 200
3. `$.message == "Logged out successfully."`

### 3.7 Logout — invalid token still succeeds (P1)
1. `POST /logout` with `{ "token": "not-valid-hex" }` → 200
2. Logout is idempotent; invalid tokens silently succeed.

### 3.8 Logout — already revoked token (P1)
1. Login → logout → logout again with same token → 200

### 3.9 Refresh — valid token rotation (P0)
1. Login → capture `refresh_token`
2. `POST /refresh` with `{ "token": "{refresh_token}" }` → 200
3. Capture new `access_token` and `refresh_token`
4. Old refresh token is now revoked (use it again → 401)

### 3.10 Refresh — invalid token (P0)
1. `POST /refresh` with `{ "token": "deadbeef" }` → 401

### 3.11 Refresh — revoked token (P1)
1. Login → logout → `POST /refresh` with same token → 401

### 3.12 Refresh — expired token (P1)
1. Login → manually expire token in DB → `POST /refresh` → 401
   (Note: requires DB manipulation; may need test-only endpoint)

### 3.13 Refresh — inactive account (P1)
1. Login → deactivate account → `POST /refresh` → 403

### 3.14 Me — authenticated (P0)
1. Login → capture `access_token`
2. `GET /me` with `Authorization: Bearer {access_token}` → 200
3. Assert response has `id`, `email`, `role == "owner"`, `is_active == true`, `email_verified_at` is non-null, `created_at`, `updated_at`

### 3.15 Me — no auth header (P0)
1. `GET /me` without Authorization → 401

### 3.16 Me — invalid token (P0)
1. `GET /me` with `Authorization: Bearer invalid.jwt.token` → 401

### 3.17 Me — expired token (P2)
1. Login → wait for expiry → `GET /me` → 401
   (Note: default TTL 900s; impractical in E2E without short TTL config)

### 3.18 Me — empty Bearer (P1)
1. `GET /me` with `Authorization: Bearer ` → 401

### 3.19 Me — wrong scheme (P2)
1. `GET /me` with `Authorization: Basic abc123` → 401

---

## Flow 4: Dashboard Invitations

**Endpoints**: `POST /api/v1/dashboard/invitations`

### 4.1 Happy path — invite staff (P0)
1. Setup: register owner → verify → login → capture `access_token`
2. `POST /invitations` with `{ "email": "alice@example.com", "role": "staff" }`, Bearer auth
3. Assert 200, `$.message` contains "alice@example.com", `$.email == "alice@example.com"`, `$.role == "staff"`

### 4.2 Re-invite same email updates role (P0)
1. Invite alice as staff → 200
2. Invite alice again as manager → 200
3. Assert `$.role == "manager"`

### 4.3 Reject owner role invitation (P0)
1. `POST /invitations` with `{ "email": "bob@example.com", "role": "owner" }` → 400
2. `$.message == "Cannot invite with owner role"`

### 4.4 Unauthenticated invitation (P0)
1. `POST /invitations` without Authorization header → 401

### 4.5 Invalid auth token (P0)
1. `POST /invitations` with `Authorization: InvalidToken` → 401

### 4.6 Invite already-registered employee (P1)
1. Register alice as employee (via service or existing flow)
2. `POST /invitations` with alice's email → 409
3. `$.message == "Email is already registered as an employee"`

### 4.7 Missing fields (P1)
1. `POST /invitations` with `{}` → 400

---

## Flow 5: Security & Edge Cases

### 5.1 SQL injection in email field (P1)
1. `POST /register` with `{ "email": "' OR 1=1 --", "password": "password1234" }`
2. Should either 201 (registers literally) or 400 (validation). Must NOT return data from other rows.

### 5.2 XSS payload in fields (P2)
1. `POST /register` with `{ "email": "<script>alert(1)</script>", "password": "password1234" }`
2. Verify response does not execute or reflect unescaped script.

### 5.3 Rate limiting — login (P2)
1. Send login requests exceeding burst config (default: 3 burst)
2. Expect 429 Too Many Requests after threshold
   (Note: requires `RATE_LIMIT_ENABLED=true` and low burst config)

### 5.4 Oversized password (P2)
1. `POST /register` with very long password (e.g., 10,000 chars)
2. Verify server doesn't hang (bcrypt cost factor should limit impact)

### 5.5 Concurrent registration (P2)
1. Two simultaneous `POST /register` requests
2. Only one should succeed with 201; other should get 409

---

## Test Data Requirements

| Data | Description | Cleanup |
|------|-------------|---------|
| Owner account | `owner@example.com` / `password1234` | Reset DB between test suites |
| Access token | Captured from login response | Auto-expires |
| Refresh token | Captured from login response | Revoked on logout/refresh |
| Verification token | From register response (hex-encoded) | Auto-deleted on verify |

## Existing Test Coverage

- **Unit tests**: `service_impl.rs` — extensive coverage for register, login, logout, refresh, verify_email, resend_verification_email
- **E2E tests**: `tests/hurl_e2e/dashboard_invitations/invite.hurl` — invitation happy paths and error cases
- **Missing E2E**: No Hurl tests for health, auth flows (register/login/logout/refresh/me/verify-email)

## Recommended Hurl Test Files

```
tests/hurl_e2e/
├── hurl.env
├── health/
│   └── health.hurl                    # Flow 1
├── dashboard_auth/
│   ├── register.hurl                  # Flow 2.1–2.5
│   ├── verify_email.hurl              # Flow 2.6–2.9
│   ├── resend_verification.hurl       # Flow 2.10–2.12
│   ├── login.hurl                     # Flow 3.1–3.5
│   ├── logout.hurl                    # Flow 3.6–3.8
│   ├── refresh.hurl                   # Flow 3.9–3.13
│   └── me.hurl                        # Flow 3.14–3.19
└── dashboard_invitations/
    └── invite.hurl                    # Flow 4 (exists, may need updates)
```

## Priority Summary

| Priority | Count | Scenarios |
|----------|-------|-----------|
| P0       | 22    | Core registration, auth, invitations, security |
| P1       | 16    | Error handling, validation, edge auth |
| P2       | 8     | Security edge cases, rate limiting, concurrency |
| **Total** | **46** | |
