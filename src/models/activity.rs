//! Activity models — definitions of activities players can do.

use spacetimedb::table;
use super::enums::ActivityKind;

/// A category grouping related Activities.
///
/// Examples: "Icebreakers", "Challenges", "Creative", etc.
#[table(name = category, public)]
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
/// A Skill is an Activity, but not every Activity is a Skill.
///
/// # Random Selection
///
/// When players roll for a random activity, the selection considers:
/// - Player preferences (configured separately)
/// - Unlocked activities (XP + prerequisites met)
/// - The `kind` field to filter Skills vs full Activities
#[table(name = activity, public)]
pub struct Activity {
    /// Unique activity identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Category this activity belongs to.
    #[index(btree)]
    pub category_id: u64,

    /// Whether this is a Skill or a full Activity.
    /// Skills can serve as prerequisites for Activities.
    #[index(btree)]
    pub kind: ActivityKind,

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
/// Activity A requires Activity B (which may be a Skill or Activity) to be completed first.
/// Since Skills are a kind of Activity, both can serve as prerequisites.
#[table(name = activity_prerequisite, public)]
pub struct ActivityPrerequisite {
    /// Unique identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The Activity that has this prerequisite.
    #[index(btree)]
    pub activity_id: u64,

    /// The Activity (or Skill) that must be completed first.
    #[index(btree)]
    pub prerequisite_id: u64,
}

