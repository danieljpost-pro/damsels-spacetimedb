//! User model — represents an authenticated user account.

use spacetimedb::{table, Identity, Timestamp};
use super::enums::UserRole;

/// An authenticated user account in the system.
///
/// Users authenticate with username + password.
/// A User can have multiple Player identities for different personas.
/// A User can be either a regular user (creates Players) or an Admin (manages system).
#[derive(Clone)]
#[table(name = user, public)]
pub struct User {
    /// Unique user identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// SpacetimeDB identity (for session tracking).
    #[unique]
    pub identity: Identity,

    /// Username for login (unique).
    #[unique]
    pub username: String,

    /// Password hash (bcrypt or similar).
    /// Stored as hex string of the hash.
    pub password_hash: String,

    /// User's role in the system.
    pub role: UserRole,

    /// When the user account was created.
    pub created_at: Timestamp,

    /// Last time the user was active.
    pub last_seen: Timestamp,
}

/// A user's preference for a specific activity category.
///
/// These preferences are the "template" that gets copied to each Player
/// when the Player is created. Users can set default preferences that
/// all their new Players will inherit.
///
/// If a user has no preferences stored, categories with ID < 100 are assumed.
#[table(name = user_category_preference, public)]
pub struct UserCategoryPreference {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The user who has this preference.
    #[index(btree)]
    pub user_id: u64,

    /// The category the user is interested in.
    #[index(btree)]
    pub category_id: u64,
}
