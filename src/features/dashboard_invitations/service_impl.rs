use std::sync::Arc;

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::service::{DashboardInvitationService, DashboardInvitationServiceError};
use crate::{
    auth,
    email::EmailSender,
    repositories::{employee_repository, invitation_repository},
};

/// Implementation of [`DashboardInvitationService`] backed by `SQLite`.
#[derive(Debug)]
pub struct DashboardInvitationServiceImpl<T: EmailSender> {
    /// Database connection pool.
    pub pool: SqlitePool,
    /// Invitation token TTL in seconds.
    pub invitation_token_ttl: u64,
    /// Email sender for invitation emails.
    pub email_sender: Arc<T>,
    /// Application name for email subjects.
    pub app_name: String,
    /// Base URL for invitation links.
    pub base_url: String,
}

impl<T: EmailSender> DashboardInvitationService for DashboardInvitationServiceImpl<T> {
    async fn invite(
        &self,
        email: &str,
        role: &str,
        invited_by: Uuid,
    ) -> Result<String, DashboardInvitationServiceError> {
        if role == "owner" {
            return Err(DashboardInvitationServiceError::OwnerRoleForbidden);
        }

        let mut txn = self.pool.begin().await?;

        // Check if email is already an employee
        let existing = employee_repository::find_by_email(&mut *txn, email)
            .await?
            .is_some();

        if existing {
            txn.rollback().await?;
            return Err(DashboardInvitationServiceError::EmailAlreadyEmployee);
        }

        // Check for existing pending invitation (re-invite)
        let is_reinvite = invitation_repository::find_pending_by_email(&mut *txn, email)
            .await?
            .is_some();

        if is_reinvite {
            invitation_repository::delete_pending_by_email(&mut *txn, email).await?;
        }
        let invitation_id = Uuid::now_v7();
        let (raw_token, token_hash) = auth::token::generate();
        let now = Utc::now();
        let ttl = i64::try_from(self.invitation_token_ttl).unwrap_or(i64::MAX);
        let expires_at = now + chrono::Duration::seconds(ttl);

        invitation_repository::insert(
            &mut *txn,
            &invitation_repository::NewInvitation {
                id: invitation_id,
                email,
                role,
                token_hash: &token_hash,
                invited_by,
                expires_at,
                created_at: now,
            },
        )
        .await?;

        txn.commit().await?;

        let token_hex = hex::encode(raw_token);

        Ok(token_hex)
    }
}
