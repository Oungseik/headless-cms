# Project Conventions

## Rust
- strictly follow rust 2024 edition pattern
- Don't use `mod.rs`, use `<name>.rs` and `<name>/` pattern instead
- run `cargo check` and `cargo fmt`

## Testing

### Unit Tests (Rust)
- Write unit tests in Rust using `#[cfg(test)]` modules within the same file as the code being tested
- Use `#[tokio::test]` for async tests
- Place tests at the bottom of the file after the implementation code
- Test function names should describe the expected behavior (e.g., `handler_should_return_ok_with_message`)

### End-to-End Tests (Hurl)
- Use Hurl for E2E testing to mimic real-world API calls from the frontend
- Place Hurl test files in `tests/hurl_e2e/` directory
- Organize tests by feature (e.g., `dashboard_auth/`, `health/`)
- Use environment variables (e.g., `{{BASE_URL}}`) for configurable endpoints
- Test both success and error scenarios
- Use `[Asserts]` to validate response bodies and status codes
- Use `[Captures]` to extract tokens or data for subsequent requests
