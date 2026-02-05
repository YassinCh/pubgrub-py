//! Error types for the resolver.

use thiserror::Error;

/// Errors that can occur during version resolution.
#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Invalid version '{version}': {reason}")]
    InvalidVersion { version: String, reason: String },

    #[error("Invalid constraint '{constraint}': {reason}")]
    InvalidConstraint { constraint: String, reason: String },

    #[error("Resolution failed: {message}")]
    ResolutionFailed { message: String, explanation: String },
}
