//! Damsels - SpacetimeDB Game Backend
//!
//! A session-based, real-time multiplayer game with an Activity Tree
//! progression system.
//!
//! ## Module Structure
//!
//! - `models` - Database table definitions (Player, Room, Activity, etc.)
//! - `reducers` - Client-callable functions organized by domain
//! - `utils` - Helper functions
//! - `admin` - Development-only admin reducers (behind `dev` feature)

// =============================================================================
// Module Declarations
// =============================================================================

pub mod models;
pub mod reducers;
pub mod utils;

/// Development-only admin module (conditionally compiled)
#[cfg(feature = "dev")]
pub mod admin;

// =============================================================================
// Re-exports
// =============================================================================

// Re-export models for SpacetimeDB table registration
pub use models::*;

// Re-export reducers (required for SpacetimeDB to find them)
pub use reducers::*;
