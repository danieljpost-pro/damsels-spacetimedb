//! Reducer modules for the Damsels SpacetimeDB module.
//!
//! Reducers are organized by domain:
//! - `lifecycle` - Client connection/disconnection handlers
//! - `player` - Player registration and sign-in
//! - `room` - Room creation, joining, and management

pub mod lifecycle;
pub mod player;
pub mod room;

// Re-export all reducers for easy access
pub use lifecycle::*;
pub use player::*;
pub use room::*;

