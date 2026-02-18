use axum::extract::FromRef;
use axum_jwt_auth::Decoder;
use serde::{Deserialize, Serialize};

use crate::api::scans::AppState;

/// JWT claims from Clerk session tokens.
///
/// Clerk session tokens use RS256 and include these standard fields.
/// The `sub` field contains the Clerk user ID (e.g., "user_2abc...").
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClerkClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub nbf: Option<usize>,
    pub azp: Option<String>,
    pub sid: Option<String>,
}

/// Newtype wrapping the Clerk user ID string extracted from JWT claims.
///
/// Use `Claims<ClerkClaims>` directly in handlers for access to all claims,
/// or use this type when you only need the user ID (via a wrapper handler).
#[derive(Debug, Clone)]
pub struct ClerkUser(pub String);

impl ClerkUser {
    /// Extracts the Clerk user ID from validated claims.
    pub fn from_claims(claims: &ClerkClaims) -> Self {
        ClerkUser(claims.sub.clone())
    }

    /// Returns the Clerk user ID string.
    pub fn user_id(&self) -> &str {
        &self.0
    }
}

/// Allows the `Claims<ClerkClaims>` extractor to locate the decoder in AppState.
///
/// axum-jwt-auth requires `Decoder<ClerkClaims>: FromRef<S>` where S is the state type.
impl FromRef<AppState> for Decoder<ClerkClaims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_decoder.clone()
    }
}
