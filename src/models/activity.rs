//! Activity models — definitions of activities players can do.

use spacetimedb::table;

/// A category grouping related Activities.
///
/// Examples: "Icebreakers", "Challenges", "Creative", etc.
#[table(name = activity_category, public)]
pub struct ActivityCategory {
    /// Unique category identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Category name.
    pub name: String,

    /// Description of this category.
    pub description: String,

    /// Display order in UI (lower = first).
    pub display_order: u32,
}

/// An Activity that players can do together.
///
/// Activities are unlocked based on XP and prerequisites.
#[table(name = activity, public)]
pub struct Activity {
    /// Unique activity identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Category this activity belongs to.
    #[index(btree)]
    pub category_id: u64,

    /// Activity name.
    pub name: String,

    /// Short description.
    pub description: String,

    /// Full instructions for the activity.
    pub instructions: String,

    /// Optional video URL demonstrating the activity.
    pub video_url: Option<String>,

    /// Minimum XP required to unlock this activity.
    pub xp_required: u64,

    /// XP awarded upon completion.
    pub xp_reward: u64,
}

/// Defines a prerequisite relationship between Activities.
///
/// Activity A requires Activity B to be completed first.
#[table(name = activity_prerequisite, public)]
pub struct ActivityPrerequisite {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The Activity that has this prerequisite.
    #[index(btree)]
    pub activity_id: u64,

    /// The Activity that must be completed first.
    #[index(btree)]
    pub prerequisite_id: u64,
}

