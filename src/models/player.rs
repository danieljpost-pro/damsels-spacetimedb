//! Player model — represents a game identity/persona.

use spacetimedb::table;
use spacetimedb::Timestamp;

/// A player identity in the Damsels game.
///
/// A User can have multiple Player identities (personas).
/// Each Player has its own display name, XP, and game progress.
/// When entering a room, a User selects which Player identity to use.
#[derive(Clone)]
#[table(name = player, public)]
pub struct Player {
    /// Unique player identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Reference to the User this player belongs to.
    /// A User can have multiple Players.
    #[index(btree)]
    pub user_id: u64,

    /// Display name for this player identity (unique across all players).
    #[unique]
    pub username: String,

    /// Total experience points accumulated by this player.
    pub xp: u64,

    /// When this player identity was created.
    pub created_at: Timestamp,
}

/// A player's preference for a specific activity category.
///
/// If a player has any preferences, only activities in preferred categories
/// will be returned. If no preferences exist, all categories are available.
#[table(name = player_category_preference, public)]
pub struct PlayerCategoryPreference {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The player who has this preference.
    #[index(btree)]
    pub player_id: u64,

    /// The category the player is interested in.
    #[index(btree)]
    pub category_id: u64,

    /// Whether this category is enabled (true) or excluded (false).
    pub enabled: bool,
}
