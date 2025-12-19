//! Reducer modules for the Damsels SpacetimeDB module.
//!
//! Reducers are organized by domain:
//! - `auth` - User registration and login
//! - `lifecycle` - Client connection/disconnection handlers
//! - `player` - Player identity management
//! - `room` - Room creation, joining, and management
//! - `activity` - PlayerActivity updates (with room-based permissions)
//! - `room_activity` - Room activity selection and completion flow
//! - `preferences` - User and Player category preferences

pub mod auth;
pub mod lifecycle;
pub mod player;
pub mod room;
pub mod activity;
pub mod room_activity;
pub mod preferences;

// Re-export all reducers for easy access
pub use auth::*;
pub use lifecycle::*;
pub use player::*;
pub use room::*;
pub use activity::*;
pub use room_activity::*;
pub use preferences::*;
