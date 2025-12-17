//! Activity Admin Reducers (DEVELOPMENT ONLY)
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
//! bypass normal authorization and allow any registered player to
//! modify the activity database.

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::activity::{activity, activity_category, activity_prerequisite};
use crate::models::player::player;
use crate::{Activity, ActivityCategory, ActivityKind, ActivityPrerequisite};

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
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    let category = ctx.db.activity_category().insert(ActivityCategory {
        id: 0,
        name: name.clone(),
        description,
        display_order,
    });

    log::info!("[DEV] Category created: {} (id: {})", name, category.id);
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
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    let category = ctx.db.activity_category().id().find(&category_id)
        .ok_or("Category not found")?;

    ctx.db.activity_category().id().update(ActivityCategory {
        name,
        description,
        display_order,
        ..category
    });

    log::info!("[DEV] Category updated: {}", category_id);
    Ok(())
}

/// Delete an activity category.
#[reducer]
pub fn admin_delete_category(ctx: &ReducerContext, category_id: u64) -> Result<(), String> {
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    // Check if any activities use this category
    let activity_count = ctx.db.activity().category_id().filter(&category_id).count();
    if activity_count > 0 {
        return Err(format!("Cannot delete category with {} activities", activity_count));
    }

    ctx.db.activity_category().id().delete(&category_id);
    log::info!("[DEV] Category deleted: {}", category_id);
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
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    // Verify category exists
    ctx.db.activity_category().id().find(&category_id)
        .ok_or("Category not found")?;

    let activity = ctx.db.activity().insert(Activity {
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
    log::info!("[DEV] {} created: {} (id: {})", kind_str, name, activity.id);
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
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    let activity = ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;

    // Verify new category exists
    ctx.db.activity_category().id().find(&category_id)
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
        ..activity
    });

    log::info!("[DEV] Activity updated: {}", activity_id);
    Ok(())
}

/// Delete an activity.
#[reducer]
pub fn admin_delete_activity(ctx: &ReducerContext, activity_id: u64) -> Result<(), String> {
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

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

    ctx.db.activity().id().delete(&activity_id);
    log::info!("[DEV] Activity deleted: {}", activity_id);
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
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

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

    log::info!("[DEV] Prerequisite added: {} requires {}", activity_id, prerequisite_id);
    Ok(())
}

/// Remove a prerequisite from an activity.
#[reducer]
pub fn admin_remove_prerequisite(
    ctx: &ReducerContext,
    prerequisite_relation_id: u64,
) -> Result<(), String> {
    let _player = ctx.db.player().identity().find(&ctx.sender)
        .ok_or("Player not registered")?;

    ctx.db.activity_prerequisite().id().delete(&prerequisite_relation_id);
    log::info!("[DEV] Prerequisite removed: {}", prerequisite_relation_id);
    Ok(())
}

