#![deny(missing_docs)]

//! Database entity definitions for the POS + CMS backend.
//!
//! Contains [`SeaQuery`](sea_query) [`Iden`](sea_query::Iden) enums for schema-aware
//! query building and [`SQLx`](sqlx) `FromRow` structs for mapping query results
//! to Rust types.

/// Employee account entity.
pub mod employee;
/// Email verification token entity.
pub mod employee_email_verification_token;
/// Refresh token entity for session management.
pub mod employee_refresh_token;
