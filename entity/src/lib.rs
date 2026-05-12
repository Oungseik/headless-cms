#![deny(missing_docs)]

//! Database entity definitions for the POS + CMS backend.
//!
//! Contains [`SeaORM`](sea_orm) entity definitions for database tables.

/// Employee account entity.
pub mod employee;
/// Email verification token entity.
pub mod employee_email_verification_token;
/// Refresh token entity for session management.
pub mod employee_refresh_token;
