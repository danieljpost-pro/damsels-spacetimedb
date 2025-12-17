//! Equipment models — items needed for activities.

use spacetimedb::table;

/// A piece of equipment that may be needed for activities.
///
/// Equipment is informational — it tells players what they need
/// to have on hand before starting an activity.
#[table(name = equipment, public)]
pub struct Equipment {
    /// Unique equipment identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// Equipment name (e.g., "Blindfold", "Rope", "Timer").
    pub name: String,

    /// Optional description or notes about this equipment.
    pub description: Option<String>,
}

/// Links an Activity to the Equipment it requires.
///
/// An Activity can require multiple pieces of Equipment,
/// and the same Equipment can be used by multiple Activities.
#[table(name = activity_equipment, public)]
pub struct ActivityEquipment {
    /// Unique identifier for this relationship.
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    /// The Activity that requires this equipment.
    #[index(btree)]
    pub activity_id: u64,

    /// The Equipment required.
    #[index(btree)]
    pub equipment_id: u64,

    /// Optional notes about how this equipment is used in this activity.
    pub notes: Option<String>,
}

