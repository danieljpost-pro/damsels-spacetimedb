//! Admin Reducers (DEVELOPMENT ONLY)
//!
//! This module is only compiled when the `dev` feature is enabled.
//! These reducers allow creating and managing activities during development.
//!
//! # Usage
//!
//! Build with dev features:
//! ```bash
//! spacetime build --features dev
//! ```
//!
//! # Security
//!
//! NEVER enable the `dev` feature in production builds. These reducers
//! allow Admin users to modify the activity database.

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::user::{user, User};
use crate::models::activity::{activity, category, activity_prerequisite};
use crate::models::equipment::{activity_equipment, equipment};
use crate::models::room::{room, room_member};
use crate::models::player::player;
use crate::{Activity, ActivityCategory, ActivityEquipment, ActivityKind, ActivityPrerequisite, Equipment, UserRole};

// =============================================================================
// Helper Functions
// =============================================================================

/// Get or create an Admin user for the current identity.
/// In dev mode, any new identity becomes an Admin with a generated username.
fn get_or_create_admin(ctx: &ReducerContext) -> Result<User, String> {
    if let Some(existing) = ctx.db.user().identity().find(&ctx.sender) {
        if existing.role != UserRole::Admin {
            return Err("User is not an Admin".to_string());
        }
        // Update last_seen
        ctx.db.user().id().update(User {
            last_seen: ctx.timestamp,
            ..existing.clone()
        });
        Ok(existing)
    } else {
        // In dev mode, create new Admin user with generated credentials
        let admin_username = format!("admin_{:?}", ctx.sender).chars().take(30).collect::<String>();
        Ok(ctx.db.user().insert(User {
            id: 0,
            identity: ctx.sender,
            username: admin_username,
            password_hash: String::new(), // No password for auto-created admins
            role: UserRole::Admin,
            created_at: ctx.timestamp,
            last_seen: ctx.timestamp,
        }))
    }
}

/// Require Admin role for the current user.
fn require_admin(ctx: &ReducerContext) -> Result<User, String> {
    let user = ctx.db.user().identity().find(&ctx.sender)
        .ok_or("User not found")?;
    
    if user.role != UserRole::Admin {
        return Err("Admin role required".to_string());
    }
    
    Ok(user)
}

// =============================================================================
// Admin Registration
// =============================================================================

/// Register or authenticate as an Admin user.
/// 
/// # Development Only
/// 
/// In dev mode, calling this will create an Admin user if one doesn't exist
/// for this identity, or verify the existing user is an Admin.
#[reducer]
pub fn admin_login(ctx: &ReducerContext) -> Result<(), String> {
    let admin = get_or_create_admin(ctx)?;
    log::info!("[ADMIN] Admin logged in: user_id={}", admin.id);
    Ok(())
}

// =============================================================================
// Category Management
// =============================================================================

/// Create a new activity category.
#[reducer]
pub fn admin_create_category(
    ctx: &ReducerContext,
    name: String,
    description: String,
    display_order: u32,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    let cat = ctx.db.category().insert(ActivityCategory {
        id: 0,
        name: name.clone(),
        description,
        display_order,
    });

    log::info!("[ADMIN] Category created: {} (id: {})", name, cat.id);
    Ok(())
}

/// Update an existing activity category.
#[reducer]
pub fn admin_update_category(
    ctx: &ReducerContext,
    category_id: u64,
    name: String,
    description: String,
    display_order: u32,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    let cat = ctx.db.category().id().find(&category_id)
        .ok_or("Category not found")?;

    ctx.db.category().id().update(ActivityCategory {
        name,
        description,
        display_order,
        ..cat
    });

    log::info!("[ADMIN] Category updated: {}", category_id);
    Ok(())
}

/// Delete an activity category.
#[reducer]
pub fn admin_delete_category(ctx: &ReducerContext, category_id: u64) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Check if any activities use this category
    let activity_count = ctx.db.activity().category_id().filter(&category_id).count();
    if activity_count > 0 {
        return Err(format!("Cannot delete category with {} activities", activity_count));
    }

    ctx.db.category().id().delete(&category_id);
    log::info!("[ADMIN] Category deleted: {}", category_id);
    Ok(())
}

// =============================================================================
// Activity Management
// =============================================================================

/// Create a new activity or skill.
#[reducer]
pub fn admin_create_activity(
    ctx: &ReducerContext,
    category_id: u64,
    kind: ActivityKind,
    name: String,
    description: String,
    instructions: String,
    video_url: Option<String>,
    xp_required: u64,
    xp_reward: u64,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Verify category exists
    ctx.db.category().id().find(&category_id)
        .ok_or("Category not found")?;

    let act = ctx.db.activity().insert(Activity {
        id: 0,
        category_id,
        kind,
        name: name.clone(),
        description,
        instructions,
        video_url,
        xp_required,
        xp_reward,
    });

    let kind_str = match kind {
        ActivityKind::Skill => "Skill",
        ActivityKind::Activity => "Activity",
    };
    log::info!("[ADMIN] {} created: {} (id: {})", kind_str, name, act.id);
    Ok(())
}

/// Update an existing activity or skill.
#[reducer]
pub fn admin_update_activity(
    ctx: &ReducerContext,
    activity_id: u64,
    category_id: u64,
    kind: ActivityKind,
    name: String,
    description: String,
    instructions: String,
    video_url: Option<String>,
    xp_required: u64,
    xp_reward: u64,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    let act = ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;

    // Verify new category exists
    ctx.db.category().id().find(&category_id)
        .ok_or("Category not found")?;

    ctx.db.activity().id().update(Activity {
        category_id,
        kind,
        name,
        description,
        instructions,
        video_url,
        xp_required,
        xp_reward,
        ..act
    });

    log::info!("[ADMIN] Activity updated: {}", activity_id);
    Ok(())
}

/// Delete an activity.
#[reducer]
pub fn admin_delete_activity(ctx: &ReducerContext, activity_id: u64) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Delete all prerequisites involving this activity
    let prereqs: Vec<_> = ctx.db.activity_prerequisite()
        .activity_id()
        .filter(&activity_id)
        .collect();
    for prereq in prereqs {
        ctx.db.activity_prerequisite().id().delete(&prereq.id);
    }

    let depending: Vec<_> = ctx.db.activity_prerequisite()
        .prerequisite_id()
        .filter(&activity_id)
        .collect();
    for prereq in depending {
        ctx.db.activity_prerequisite().id().delete(&prereq.id);
    }

    // Delete all equipment links for this activity
    let equip_links: Vec<_> = ctx.db.activity_equipment()
        .activity_id()
        .filter(&activity_id)
        .collect();
    for link in equip_links {
        ctx.db.activity_equipment().id().delete(&link.id);
    }

    ctx.db.activity().id().delete(&activity_id);
    log::info!("[ADMIN] Activity deleted: {}", activity_id);
    Ok(())
}

// =============================================================================
// Prerequisite Management
// =============================================================================

/// Add a prerequisite to an activity.
#[reducer]
pub fn admin_add_prerequisite(
    ctx: &ReducerContext,
    activity_id: u64,
    prerequisite_id: u64,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Verify both activities exist
    ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    ctx.db.activity().id().find(&prerequisite_id)
        .ok_or("Prerequisite activity not found")?;

    // Prevent self-reference
    if activity_id == prerequisite_id {
        return Err("Activity cannot be its own prerequisite".to_string());
    }

    // Check if prerequisite already exists
    let existing = ctx.db.activity_prerequisite()
        .activity_id()
        .filter(&activity_id)
        .find(|p| p.prerequisite_id == prerequisite_id);
    
    if existing.is_some() {
        return Err("Prerequisite already exists".to_string());
    }

    ctx.db.activity_prerequisite().insert(ActivityPrerequisite {
        id: 0,
        activity_id,
        prerequisite_id,
    });

    log::info!("[ADMIN] Prerequisite added: {} requires {}", activity_id, prerequisite_id);
    Ok(())
}

/// Remove a prerequisite from an activity.
#[reducer]
pub fn admin_remove_prerequisite(
    ctx: &ReducerContext,
    prerequisite_relation_id: u64,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    ctx.db.activity_prerequisite().id().delete(&prerequisite_relation_id);
    log::info!("[ADMIN] Prerequisite removed: {}", prerequisite_relation_id);
    Ok(())
}

// =============================================================================
// Equipment Management
// =============================================================================

/// Create a new piece of equipment.
#[reducer]
pub fn admin_create_equipment(
    ctx: &ReducerContext,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    let equip = ctx.db.equipment().insert(Equipment {
        id: 0,
        name: name.clone(),
        description,
    });

    log::info!("[ADMIN] Equipment created: {} (id: {})", name, equip.id);
    Ok(())
}

/// Update an existing piece of equipment.
#[reducer]
pub fn admin_update_equipment(
    ctx: &ReducerContext,
    equipment_id: u64,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    let equip = ctx.db.equipment().id().find(&equipment_id)
        .ok_or("Equipment not found")?;

    ctx.db.equipment().id().update(Equipment {
        name,
        description,
        ..equip
    });

    log::info!("[ADMIN] Equipment updated: {}", equipment_id);
    Ok(())
}

/// Delete a piece of equipment.
#[reducer]
pub fn admin_delete_equipment(ctx: &ReducerContext, equipment_id: u64) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Remove all activity-equipment links for this equipment
    let links: Vec<_> = ctx.db.activity_equipment()
        .equipment_id()
        .filter(&equipment_id)
        .collect();
    for link in links {
        ctx.db.activity_equipment().id().delete(&link.id);
    }

    ctx.db.equipment().id().delete(&equipment_id);
    log::info!("[ADMIN] Equipment deleted: {}", equipment_id);
    Ok(())
}

/// Add equipment requirement to an activity.
#[reducer]
pub fn admin_add_activity_equipment(
    ctx: &ReducerContext,
    activity_id: u64,
    equipment_id: u64,
    notes: Option<String>,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Verify activity exists
    ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;

    // Verify equipment exists
    ctx.db.equipment().id().find(&equipment_id)
        .ok_or("Equipment not found")?;

    // Check if link already exists
    let existing = ctx.db.activity_equipment()
        .activity_id()
        .filter(&activity_id)
        .find(|ae| ae.equipment_id == equipment_id);

    if existing.is_some() {
        return Err("Activity already has this equipment".to_string());
    }

    ctx.db.activity_equipment().insert(ActivityEquipment {
        id: 0,
        activity_id,
        equipment_id,
        notes,
    });

    log::info!("[ADMIN] Equipment {} added to activity {}", equipment_id, activity_id);
    Ok(())
}

/// Remove equipment requirement from an activity.
#[reducer]
pub fn admin_remove_activity_equipment(
    ctx: &ReducerContext,
    activity_equipment_id: u64,
) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    ctx.db.activity_equipment().id().delete(&activity_equipment_id);
    log::info!("[ADMIN] Activity equipment removed: {}", activity_equipment_id);
    Ok(())
}

// =============================================================================
// Data Management (DEV ONLY)
// =============================================================================

/// Delete a player and their room memberships.
#[reducer]
pub fn admin_delete_player(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Delete all room memberships for this player
    let memberships: Vec<_> = ctx.db.room_member().iter()
        .filter(|m| m.player_id == player_id)
        .collect();
    
    for m in memberships {
        ctx.db.room_member().id().delete(&m.id);
    }
    
    // Delete the player
    ctx.db.player().id().delete(&player_id);
    log::info!("[ADMIN] Player deleted: {}", player_id);
    Ok(())
}

/// Delete a room and all its members.
#[reducer]
pub fn admin_delete_room(ctx: &ReducerContext, room_id: u64) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Delete all room memberships
    let memberships: Vec<_> = ctx.db.room_member().room_id().filter(&room_id).collect();
    
    for m in memberships {
        ctx.db.room_member().id().delete(&m.id);
    }
    
    // Delete the room
    ctx.db.room().id().delete(&room_id);
    log::info!("[ADMIN] Room deleted: {}", room_id);
    Ok(())
}

/// Clear all players, rooms, and memberships (nuclear option for dev testing).
#[reducer]
pub fn admin_clear_all_players_and_rooms(ctx: &ReducerContext) -> Result<(), String> {
    let _admin = require_admin(ctx)?;

    // Delete all room memberships
    let memberships: Vec<_> = ctx.db.room_member().iter().collect();
    for m in memberships {
        ctx.db.room_member().id().delete(&m.id);
    }
    
    // Delete all rooms
    let rooms: Vec<_> = ctx.db.room().iter().collect();
    for r in rooms {
        ctx.db.room().id().delete(&r.id);
    }
    
    // Delete all players
    let players: Vec<_> = ctx.db.player().iter().collect();
    for p in players {
        ctx.db.player().id().delete(&p.id);
    }
    
    log::info!("[ADMIN] All players and rooms cleared");
    Ok(())
}

// =============================================================================
// Seed Data Reducers (No Player Required - Creates Admin User)
// =============================================================================

/// Seed a category. Creates Admin user if needed.
/// 
/// # Development Only
/// 
/// This reducer creates an Admin user for the caller if one doesn't exist.
#[reducer]
pub fn seed_category(
    ctx: &ReducerContext,
    name: String,
    description: String,
    display_order: u32,
) -> Result<(), String> {
    let _admin = get_or_create_admin(ctx)?;

    let cat = ctx.db.category().insert(ActivityCategory {
        id: 0,
        name: name.clone(),
        description,
        display_order,
    });
    log::info!("[SEED] Category created: {} (id: {})", name, cat.id);
    Ok(())
}

/// Seed equipment. Creates Admin user if needed.
/// 
/// # Development Only
#[reducer]
pub fn seed_equipment(
    ctx: &ReducerContext,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let _admin = get_or_create_admin(ctx)?;

    let equip = ctx.db.equipment().insert(Equipment {
        id: 0,
        name: name.clone(),
        description,
    });
    log::info!("[SEED] Equipment created: {} (id: {})", name, equip.id);
    Ok(())
}

/// Seed an activity. Creates Admin user if needed.
/// 
/// # Development Only
#[reducer]
pub fn seed_activity(
    ctx: &ReducerContext,
    category_id: u64,
    kind: ActivityKind,
    name: String,
    description: String,
    instructions: String,
    video_url: Option<String>,
    xp_required: u64,
    xp_reward: u64,
) -> Result<(), String> {
    let _admin = get_or_create_admin(ctx)?;

    ctx.db.category().id().find(&category_id)
        .ok_or("Category not found")?;

    let act = ctx.db.activity().insert(Activity {
        id: 0,
        category_id,
        kind,
        name: name.clone(),
        description,
        instructions,
        video_url,
        xp_required,
        xp_reward,
    });
    log::info!("[SEED] Activity created: {} (id: {})", name, act.id);
    Ok(())
}

/// Seed a prerequisite relationship. Creates Admin user if needed.
/// 
/// # Development Only
#[reducer]
pub fn seed_prerequisite(
    ctx: &ReducerContext,
    activity_id: u64,
    prerequisite_id: u64,
) -> Result<(), String> {
    let _admin = get_or_create_admin(ctx)?;

    ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    ctx.db.activity().id().find(&prerequisite_id)
        .ok_or("Prerequisite activity not found")?;

    ctx.db.activity_prerequisite().insert(ActivityPrerequisite {
        id: 0,
        activity_id,
        prerequisite_id,
    });
    log::info!("[SEED] Prerequisite: {} requires {}", activity_id, prerequisite_id);
    Ok(())
}

/// Seed activity-equipment link. Creates Admin user if needed.
/// 
/// # Development Only
#[reducer]
pub fn seed_activity_equipment(
    ctx: &ReducerContext,
    activity_id: u64,
    equipment_id: u64,
    notes: Option<String>,
) -> Result<(), String> {
    let _admin = get_or_create_admin(ctx)?;

    ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    ctx.db.equipment().id().find(&equipment_id)
        .ok_or("Equipment not found")?;

    ctx.db.activity_equipment().insert(ActivityEquipment {
        id: 0,
        activity_id,
        equipment_id,
        notes,
    });
    log::info!("[SEED] Equipment {} linked to activity {}", equipment_id, activity_id);
    Ok(())
}
