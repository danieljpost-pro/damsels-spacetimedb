//! Preference reducers for managing User and Player category preferences.
//!
//! Users have preferences that act as templates for new Players.
//! When a Player is created, the User's preferences are copied to the Player.
//! If no preferences are set, categories with ID < 100 are assumed as defaults.

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::user::{user_category_preference, UserCategoryPreference};
use crate::models::player::{player_category_preference, PlayerCategoryPreference, player};
use crate::models::activity::category;
use crate::reducers::auth::require_user;

/// Default category ID threshold - categories with ID < 100 are considered "safe" defaults
const DEFAULT_CATEGORY_THRESHOLD: u64 = 100;

/// Initialize user preferences with default categories (ID < 100).
/// Called automatically when the user first opens the preferences dialog.
#[reducer]
pub fn init_user_preferences(ctx: &ReducerContext) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    // Check if user already has preferences
    let existing_prefs: Vec<_> = ctx.db.user_category_preference()
        .user_id()
        .filter(&user.id)
        .collect();
    
    if !existing_prefs.is_empty() {
        // Already has preferences, no need to init
        return Ok(());
    }
    
    // Get all categories with ID < 100 and create preferences for them
    let default_categories: Vec<_> = ctx.db.category()
        .iter()
        .filter(|c| c.id < DEFAULT_CATEGORY_THRESHOLD)
        .collect();
    
    for cat in default_categories {
        ctx.db.user_category_preference().insert(UserCategoryPreference {
            id: 0,
            user_id: user.id,
            category_id: cat.id,
        });
    }
    
    log::info!("Initialized preferences for user {} with default categories", user.id);
    Ok(())
}

/// Add a category to user preferences.
#[reducer]
pub fn add_user_category_preference(
    ctx: &ReducerContext,
    category_id: u64,
) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    // Verify category exists
    ctx.db.category().id().find(&category_id)
        .ok_or("Category not found")?;
    
    // Check if already exists
    let existing = ctx.db.user_category_preference()
        .user_id()
        .filter(&user.id)
        .find(|p| p.category_id == category_id);
    
    if existing.is_some() {
        return Ok(()); // Already exists, no-op
    }
    
    ctx.db.user_category_preference().insert(UserCategoryPreference {
        id: 0,
        user_id: user.id,
        category_id,
    });
    
    log::info!("User {} added category {} to preferences", user.id, category_id);
    Ok(())
}

/// Remove a category from user preferences.
#[reducer]
pub fn remove_user_category_preference(
    ctx: &ReducerContext,
    category_id: u64,
) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    let pref = ctx.db.user_category_preference()
        .user_id()
        .filter(&user.id)
        .find(|p| p.category_id == category_id);
    
    if let Some(p) = pref {
        ctx.db.user_category_preference().id().delete(&p.id);
        log::info!("User {} removed category {} from preferences", user.id, category_id);
    }
    
    Ok(())
}

/// Set all user category preferences at once (replaces existing).
/// This is more efficient when making multiple changes.
#[reducer]
pub fn set_user_category_preferences(
    ctx: &ReducerContext,
    category_ids: Vec<u64>,
) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    // Validate all categories exist
    for cat_id in &category_ids {
        ctx.db.category().id().find(cat_id)
            .ok_or(format!("Category {} not found", cat_id))?;
    }
    
    // Remove all existing preferences
    let existing: Vec<_> = ctx.db.user_category_preference()
        .user_id()
        .filter(&user.id)
        .collect();
    
    for pref in existing {
        ctx.db.user_category_preference().id().delete(&pref.id);
    }
    
    // Add new preferences
    for cat_id in &category_ids {
        ctx.db.user_category_preference().insert(UserCategoryPreference {
            id: 0,
            user_id: user.id,
            category_id: *cat_id,
        });
    }
    
    log::info!("User {} set {} category preferences", user.id, category_ids.len());
    Ok(())
}

/// Initialize player preferences by copying from user preferences.
/// Called automatically when a player is created.
pub fn copy_user_prefs_to_player(ctx: &ReducerContext, user_id: u64, player_id: u64) {
    // Get user preferences
    let user_prefs: Vec<_> = ctx.db.user_category_preference()
        .user_id()
        .filter(&user_id)
        .collect();
    
    if user_prefs.is_empty() {
        // No user preferences, use defaults (categories with ID < 100)
        let default_categories: Vec<_> = ctx.db.category()
            .iter()
            .filter(|c| c.id < DEFAULT_CATEGORY_THRESHOLD)
            .collect();
        
        for cat in default_categories {
            ctx.db.player_category_preference().insert(PlayerCategoryPreference {
                id: 0,
                player_id,
                category_id: cat.id,
            });
        }
        log::info!("Player {} initialized with default category preferences", player_id);
    } else {
        // Copy user preferences to player
        for pref in user_prefs {
            ctx.db.player_category_preference().insert(PlayerCategoryPreference {
                id: 0,
                player_id,
                category_id: pref.category_id,
            });
        }
        log::info!("Player {} inherited {} category preferences from user {}", 
            player_id, ctx.db.player_category_preference().player_id().filter(&player_id).count(), user_id);
    }
}

/// Add a category to player preferences.
#[reducer]
pub fn add_player_category_preference(
    ctx: &ReducerContext,
    player_id: u64,
    category_id: u64,
) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    // Verify player belongs to user
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("Player does not belong to you".to_string());
    }
    
    // Verify category exists
    ctx.db.category().id().find(&category_id)
        .ok_or("Category not found")?;
    
    // Check if already exists
    let existing = ctx.db.player_category_preference()
        .player_id()
        .filter(&player_id)
        .find(|p| p.category_id == category_id);
    
    if existing.is_some() {
        return Ok(()); // Already exists, no-op
    }
    
    ctx.db.player_category_preference().insert(PlayerCategoryPreference {
        id: 0,
        player_id,
        category_id,
    });
    
    log::info!("Player {} added category {} to preferences", player_id, category_id);
    Ok(())
}

/// Remove a category from player preferences.
#[reducer]
pub fn remove_player_category_preference(
    ctx: &ReducerContext,
    player_id: u64,
    category_id: u64,
) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    // Verify player belongs to user
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("Player does not belong to you".to_string());
    }
    
    let pref = ctx.db.player_category_preference()
        .player_id()
        .filter(&player_id)
        .find(|p| p.category_id == category_id);
    
    if let Some(p) = pref {
        ctx.db.player_category_preference().id().delete(&p.id);
        log::info!("Player {} removed category {} from preferences", player_id, category_id);
    }
    
    Ok(())
}

/// Set all player category preferences at once (replaces existing).
#[reducer]
pub fn set_player_category_preferences(
    ctx: &ReducerContext,
    player_id: u64,
    category_ids: Vec<u64>,
) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    // Verify player belongs to user
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("Player does not belong to you".to_string());
    }
    
    // Validate all categories exist
    for cat_id in &category_ids {
        ctx.db.category().id().find(cat_id)
            .ok_or(format!("Category {} not found", cat_id))?;
    }
    
    // Remove all existing preferences
    let existing: Vec<_> = ctx.db.player_category_preference()
        .player_id()
        .filter(&player_id)
        .collect();
    
    for pref in existing {
        ctx.db.player_category_preference().id().delete(&pref.id);
    }
    
    // Add new preferences
    for cat_id in &category_ids {
        ctx.db.player_category_preference().insert(PlayerCategoryPreference {
            id: 0,
            player_id,
            category_id: *cat_id,
        });
    }
    
    log::info!("Player {} set {} category preferences", player_id, category_ids.len());
    Ok(())
}

/// Request a refresh of the category subscription.
/// 
/// This reducer serves as a synchronization point - calling it successfully
/// confirms the WebSocket connection is active and subscriptions should be
/// populated. The actual category data comes through the subscription.
/// 
/// Logs the category count for debugging purposes.
#[reducer]
pub fn refresh_categories(ctx: &ReducerContext) -> Result<(), String> {
    let _user = require_user(ctx)?;
    
    let count = ctx.db.category().iter().count();
    let default_count = ctx.db.category().iter().filter(|c| c.id < DEFAULT_CATEGORY_THRESHOLD).count();
    let adult_count = count - default_count;
    
    log::info!("[Categories] Total: {}, Default (ID<100): {}, Adult (ID>=100): {}", 
        count, default_count, adult_count);
    
    Ok(())
}

/// Get effective category IDs for a player.
/// If the player has preferences, returns those.
/// Otherwise returns categories with ID < 100 as defaults.
pub fn get_player_effective_categories(ctx: &ReducerContext, player_id: u64) -> Vec<u64> {
    let prefs: Vec<u64> = ctx.db.player_category_preference()
        .player_id()
        .filter(&player_id)
        .map(|p| p.category_id)
        .collect();
    
    if prefs.is_empty() {
        // Return defaults
        ctx.db.category()
            .iter()
            .filter(|c| c.id < DEFAULT_CATEGORY_THRESHOLD)
            .map(|c| c.id)
            .collect()
    } else {
        prefs
    }
}

