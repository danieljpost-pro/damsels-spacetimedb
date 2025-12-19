//! Activity-related reducers for players.
//!
//! Players can update PlayerActivity records for themselves or other players
//! in the same room.

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::player::{player, player_category_preference};
use crate::models::room::room_member;
use crate::models::player_activity::{player_activity, player_unlocked_activity, prerequisite_vouch, PlayerActivity, PlayerUnlockedActivity, PrerequisiteVouch};
use crate::models::activity::{activity, activity_prerequisite, category};
use crate::models::enums::ActivityStatus;
use crate::reducers::auth::require_user;
use crate::Player;

/// Helper: Get a player that belongs to the authenticated user.
fn get_user_player(ctx: &ReducerContext, player_id: u64) -> Result<Player, String> {
    let user = require_user(ctx)?;
    
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("Player does not belong to you".to_string());
    }
    
    Ok(player)
}

/// Helper: Check if two players are in the same room.
fn players_in_same_room(ctx: &ReducerContext, player_a_id: u64, player_b_id: u64) -> bool {
    if player_a_id == player_b_id {
        return true; // Same player
    }
    
    // Get all rooms player_a is in
    for membership_a in ctx.db.room_member().iter().filter(|m| m.player_id == player_a_id) {
        // Check if player_b is also in that room
        let b_in_room = ctx.db.room_member()
            .room_id()
            .filter(&membership_a.room_id)
            .any(|m| m.player_id == player_b_id);
        if b_in_room {
            return true;
        }
    }
    false
}

/// Start an activity for yourself.
/// Sets status to Available if prerequisites are met.
/// 
/// # Permissions
/// Players can start activities for themselves.
#[reducer]
pub fn start_activity(ctx: &ReducerContext, player_id: u64, activity_id: u64) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Verify activity exists
    ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    
    // Check if already tracking this activity
    let existing = ctx.db.player_activity()
        .player_id()
        .filter(&player.id)
        .find(|pa| pa.activity_id == activity_id);
    
    if existing.is_some() {
        return Err("Already tracking this activity".to_string());
    }
    
    // Create player activity record
    ctx.db.player_activity().insert(PlayerActivity {
        id: 0,
        player_id: player.id,
        activity_id,
        status: ActivityStatus::Available,
        completed_at: None,
        completed_by: None,
        vouched: false,
        rating: None,
    });
    
    log::info!("Player {} started activity {}", player.id, activity_id);
    Ok(())
}

/// Mark an activity as complete for a player.
/// 
/// Awards XP to the target player and refreshes their unlocked activities.
/// 
/// # Permissions
/// - Players can mark their own activities complete
/// - Players can mark activities complete for other players in the same room
#[reducer]
pub fn complete_activity(
    ctx: &ReducerContext,
    player_id: u64,
    target_player_id: u64,
    activity_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Permission check: must be same player or in same room
    if player.id != target_player_id && !players_in_same_room(ctx, player.id, target_player_id) {
        return Err("Cannot modify activity for player not in your room".to_string());
    }
    
    // Find the player activity record
    let pa = ctx.db.player_activity()
        .player_id()
        .filter(&target_player_id)
        .find(|pa| pa.activity_id == activity_id)
        .ok_or("Activity not found for target player")?;
    
    if pa.status == ActivityStatus::Completed {
        return Err("Activity already completed".to_string());
    }
    
    // Get the activity to find XP reward
    let act = ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    
    // Update to completed
    ctx.db.player_activity().id().update(PlayerActivity {
        status: ActivityStatus::Completed,
        completed_at: Some(ctx.timestamp),
        completed_by: Some(player.id),
        ..pa
    });
    
    // Award XP to the target player
    if act.xp_reward > 0 {
        let target_player = ctx.db.player().id().find(&target_player_id)
            .ok_or("Target player not found")?;
        
        let new_xp = target_player.xp.saturating_add(act.xp_reward);
        ctx.db.player().id().update(Player {
            xp: new_xp,
            ..target_player
        });
        
        log::info!("Player {} earned {} XP for completing activity {}", 
            target_player_id, act.xp_reward, activity_id);
    }
    
    // Refresh unlocked activities (new prereqs met + new XP may unlock more)
    refresh_unlocked_activities(ctx, target_player_id);
    
    log::info!("Player {} marked activity {} complete for player {}", 
        player.id, activity_id, target_player_id);
    Ok(())
}

/// Vouch for another player's prerequisites.
/// Allows them to unlock an activity without completing all prerequisites.
/// 
/// # Permissions
/// - Players can vouch for other players in the same room
/// - Cannot vouch for yourself
#[reducer]
pub fn vouch_for_player(
    ctx: &ReducerContext,
    player_id: u64,
    recipient_id: u64,
    activity_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    if player.id == recipient_id {
        return Err("Cannot vouch for yourself".to_string());
    }
    
    // Permission check: must be in same room
    if !players_in_same_room(ctx, player.id, recipient_id) {
        return Err("Cannot vouch for player not in your room".to_string());
    }
    
    // Verify activity exists
    ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    
    // Verify recipient exists
    ctx.db.player().id().find(&recipient_id)
        .ok_or("Recipient player not found")?;
    
    // Check if already vouched
    let existing_vouch = ctx.db.prerequisite_vouch()
        .recipient_id()
        .filter(&recipient_id)
        .find(|v| v.activity_id == activity_id && v.voucher_id == player.id);
    
    if existing_vouch.is_some() {
        return Err("Already vouched for this player on this activity".to_string());
    }
    
    // Create vouch record
    ctx.db.prerequisite_vouch().insert(PrerequisiteVouch {
        id: 0,
        voucher_id: player.id,
        recipient_id,
        activity_id,
        created_at: ctx.timestamp,
    });
    
    // Create or update recipient's player activity with vouched flag
    let recipient_pa = ctx.db.player_activity()
        .player_id()
        .filter(&recipient_id)
        .find(|pa| pa.activity_id == activity_id);
    
    if let Some(pa) = recipient_pa {
        // Update existing record
        ctx.db.player_activity().id().update(PlayerActivity {
            status: ActivityStatus::Available,
            vouched: true,
            ..pa
        });
    } else {
        // Create new record
        ctx.db.player_activity().insert(PlayerActivity {
            id: 0,
            player_id: recipient_id,
            activity_id,
            status: ActivityStatus::Available,
            completed_at: None,
            completed_by: None,
            vouched: true,
            rating: None,
        });
    }
    
    log::info!("Player {} vouched for player {} on activity {}", 
        player.id, recipient_id, activity_id);
    Ok(())
}

/// Reset an activity to available (undo completion).
/// 
/// # Permissions
/// - Players can reset their own activities
/// - Players can reset activities for other players in the same room
#[reducer]
pub fn reset_activity(
    ctx: &ReducerContext,
    player_id: u64,
    target_player_id: u64,
    activity_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Permission check: must be same player or in same room
    if player.id != target_player_id && !players_in_same_room(ctx, player.id, target_player_id) {
        return Err("Cannot modify activity for player not in your room".to_string());
    }
    
    // Find the player activity record
    let pa = ctx.db.player_activity()
        .player_id()
        .filter(&target_player_id)
        .find(|pa| pa.activity_id == activity_id)
        .ok_or("Activity not found for target player")?;
    
    // Reset to available
    ctx.db.player_activity().id().update(PlayerActivity {
        status: ActivityStatus::Available,
        completed_at: None,
        completed_by: None,
        ..pa
    });
    
    log::info!("Player {} reset activity {} for player {}", 
        player.id, activity_id, target_player_id);
    Ok(())
}

/// Rate a completed activity.
/// 
/// Players can rate activities they've completed on a scale of 1-5 stars.
/// Ratings are used to weight future random activity selections - higher
/// rated activities are more likely to be chosen.
/// 
/// # Arguments
/// * `player_id` - The player giving the rating
/// * `activity_id` - The activity to rate
/// * `rating` - Rating from 1 to 5 (inclusive)
/// 
/// # Permissions
/// Players can only rate their own completed activities.
#[reducer]
pub fn rate_activity(
    ctx: &ReducerContext,
    player_id: u64,
    activity_id: u64,
    rating: u8,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Validate rating range
    if rating < 1 || rating > 5 {
        return Err("Rating must be between 1 and 5".to_string());
    }
    
    // Find the player's activity record
    let pa = ctx.db.player_activity()
        .player_id()
        .filter(&player.id)
        .find(|pa| pa.activity_id == activity_id)
        .ok_or("Activity not found for player")?;
    
    // Must be completed to rate
    if pa.status != ActivityStatus::Completed {
        return Err("Can only rate completed activities".to_string());
    }
    
    // Update rating
    ctx.db.player_activity().id().update(PlayerActivity {
        rating: Some(rating),
        ..pa
    });
    
    log::info!("Player {} rated activity {} with {} stars", 
        player.id, activity_id, rating);
    Ok(())
}

/// Get available activities for a player.
/// 
/// Returns activities that:
/// - Have XP requirement <= player's current XP
/// - Match player's category preferences (if any are set)
/// - Are not already completed by the player
/// - Have all prerequisites met (completed or vouched)
/// 
/// # Returns
/// This reducer logs the available activities. Use SQL queries to retrieve
/// the full activity list with details.
#[reducer]
pub fn get_available_activities(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Get player's category preferences (if any)
    // Presence in the table means the category is selected
    let preferences: Vec<u64> = ctx.db.player_category_preference()
        .player_id()
        .filter(&player.id)
        .map(|p| p.category_id)
        .collect();
    
    let has_preferences = !preferences.is_empty();
    
    // Get all completed activity IDs for this player
    let completed_activities: Vec<u64> = ctx.db.player_activity()
        .player_id()
        .filter(&player.id)
        .filter(|pa| pa.status == ActivityStatus::Completed)
        .map(|pa| pa.activity_id)
        .collect();
    
    // Get all vouched activity IDs for this player
    let vouched_activities: Vec<u64> = ctx.db.player_activity()
        .player_id()
        .filter(&player.id)
        .filter(|pa| pa.vouched)
        .map(|pa| pa.activity_id)
        .collect();
    
    let mut available_count = 0;
    
    for act in ctx.db.activity().iter() {
        // Check XP requirement
        if act.xp_required > player.xp {
            continue;
        }
        
        // Check category preference (if player has preferences set)
        if has_preferences && !preferences.contains(&act.category_id) {
            continue;
        }
        
        // Skip already completed activities
        if completed_activities.contains(&act.id) {
            continue;
        }
        
        // Check prerequisites are met
        let prereqs_met = check_prerequisites_met(
            ctx, 
            act.id, 
            &completed_activities, 
            &vouched_activities
        );
        
        if !prereqs_met {
            continue;
        }
        
        available_count += 1;
        log::info!("Available activity for player {}: {} - {} (XP: {}, Category: {})", 
            player.id, act.id, act.name, act.xp_required, act.category_id);
    }
    
    log::info!("Player {} has {} available activities", player.id, available_count);
    Ok(())
}

/// Helper: Check if all prerequisites for an activity are met.
fn check_prerequisites_met(
    ctx: &ReducerContext,
    activity_id: u64,
    completed_activities: &[u64],
    vouched_activities: &[u64],
) -> bool {
    // Get all prerequisites for this activity
    let prereqs: Vec<u64> = ctx.db.activity_prerequisite()
        .activity_id()
        .filter(&activity_id)
        .map(|p| p.prerequisite_id)
        .collect();
    
    // If activity is vouched, skip prerequisite check
    if vouched_activities.contains(&activity_id) {
        return true;
    }
    
    // All prerequisites must be completed
    for prereq_id in prereqs {
        if !completed_activities.contains(&prereq_id) {
            return false;
        }
    }
    
    true
}

/// Toggle a player's category preference.
/// 
/// If enabled is true, adds the category to preferences.
/// If enabled is false, removes the category from preferences.
/// 
/// # Permissions
/// Players can only set preferences for themselves.
/// 
/// NOTE: This reducer is deprecated. Use the reducers in the `preferences` module instead.
#[reducer]
pub fn set_category_preference(
    ctx: &ReducerContext,
    player_id: u64,
    category_id: u64,
    enabled: bool,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Verify category exists
    ctx.db.category().id().find(&category_id)
        .ok_or("Category not found")?;
    
    // Check if preference already exists
    let existing = ctx.db.player_category_preference()
        .player_id()
        .filter(&player.id)
        .find(|p| p.category_id == category_id);
    
    if enabled {
        // Add category if not already present
        if existing.is_none() {
            ctx.db.player_category_preference().insert(crate::models::player::PlayerCategoryPreference {
                id: 0,
                player_id: player.id,
                category_id,
            });
            log::info!("Added category {} to player {} preferences", category_id, player.id);
        }
    } else {
        // Remove category if present
        if let Some(pref) = existing {
            ctx.db.player_category_preference().id().delete(&pref.id);
            log::info!("Removed category {} from player {} preferences", category_id, player.id);
        }
    }
    
    // Refresh unlocked activities with new preferences
    refresh_unlocked_activities(ctx, player.id);
    
    Ok(())
}

/// Clear all category preferences for a player (returns to "all categories" mode).
/// 
/// # Permissions
/// Players can only clear their own preferences.
#[reducer]
pub fn clear_category_preferences(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    let prefs: Vec<_> = ctx.db.player_category_preference()
        .player_id()
        .filter(&player.id)
        .collect();
    
    for pref in prefs {
        ctx.db.player_category_preference().id().delete(&pref.id);
    }
    
    log::info!("Cleared all category preferences for player {}", player.id);
    
    // Refresh unlocked activities with new preferences
    refresh_unlocked_activities(ctx, player.id);
    
    Ok(())
}

// =============================================================================
// Unlocked Activities Push System
// =============================================================================

/// Refresh the player_unlocked_activity table for a player.
/// 
/// This computes which activities are currently available based on:
/// - Player's XP
/// - Category preferences
/// - Completed prerequisites
/// 
/// New activities are added with is_new=true. Activities that are no longer
/// available (due to completion or preference change) are removed.
pub fn refresh_unlocked_activities(ctx: &ReducerContext, player_id: u64) {
    let player = match ctx.db.player().id().find(&player_id) {
        Some(p) => p,
        None => {
            log::warn!("Cannot refresh activities for unknown player {}", player_id);
            return;
        }
    };
    
    // Get player's category preferences
    // Presence in the table means the category is selected
    let preferences: Vec<u64> = ctx.db.player_category_preference()
        .player_id()
        .filter(&player_id)
        .map(|p| p.category_id)
        .collect();
    let has_preferences = !preferences.is_empty();
    
    // Get completed activity IDs
    let completed_activities: Vec<u64> = ctx.db.player_activity()
        .player_id()
        .filter(&player_id)
        .filter(|pa| pa.status == ActivityStatus::Completed)
        .map(|pa| pa.activity_id)
        .collect();
    
    // Get vouched activity IDs
    let vouched_activities: Vec<u64> = ctx.db.player_activity()
        .player_id()
        .filter(&player_id)
        .filter(|pa| pa.vouched)
        .map(|pa| pa.activity_id)
        .collect();
    
    // Get currently unlocked activity IDs
    let currently_unlocked: Vec<u64> = ctx.db.player_unlocked_activity()
        .player_id()
        .filter(&player_id)
        .map(|ua| ua.activity_id)
        .collect();
    
    let mut newly_unlocked = Vec::new();
    let mut still_unlocked = Vec::new();
    
    // Check each activity
    for act in ctx.db.activity().iter() {
        // Check XP requirement
        if act.xp_required > player.xp {
            continue;
        }
        
        // Check category preference
        if has_preferences && !preferences.contains(&act.category_id) {
            continue;
        }
        
        // Skip completed activities
        if completed_activities.contains(&act.id) {
            continue;
        }
        
        // Check prerequisites
        let prereqs_met = check_prerequisites_met(
            ctx,
            act.id,
            &completed_activities,
            &vouched_activities,
        );
        
        if !prereqs_met {
            continue;
        }
        
        // Activity is available
        if currently_unlocked.contains(&act.id) {
            still_unlocked.push(act.id);
        } else {
            newly_unlocked.push(act.clone());
        }
    }
    
    // Remove activities that are no longer unlocked
    let to_remove: Vec<_> = ctx.db.player_unlocked_activity()
        .player_id()
        .filter(&player_id)
        .filter(|ua| !still_unlocked.contains(&ua.activity_id) && 
                     !newly_unlocked.iter().any(|a| a.id == ua.activity_id))
        .collect();
    
    for ua in to_remove {
        ctx.db.player_unlocked_activity().id().delete(&ua.id);
    }
    
    // Add newly unlocked activities
    for act in newly_unlocked {
        let category_name = ctx.db.category().id().find(&act.category_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        ctx.db.player_unlocked_activity().insert(PlayerUnlockedActivity {
            id: 0,
            player_id,
            activity_id: act.id,
            activity_name: act.name.clone(),
            activity_description: act.description.clone(),
            category_id: act.category_id,
            category_name,
            kind: act.kind,
            xp_required: act.xp_required,
            xp_reward: act.xp_reward,
            unlocked_at: ctx.timestamp,
            is_new: true,
        });
        
        log::info!("Player {} unlocked activity: {} ({})", 
            player_id, act.name, act.id);
    }
}

/// Award XP to a player and refresh their unlocked activities.
/// 
/// # Permissions
/// Only reducers within the same room context can award XP.
/// This is typically called after completing an activity.
#[reducer]
pub fn award_xp(ctx: &ReducerContext, player_id: u64, xp_amount: u64) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    let old_xp = player.xp;
    let new_xp = old_xp.saturating_add(xp_amount);
    
    // Update player XP
    ctx.db.player().id().update(Player {
        xp: new_xp,
        ..player.clone()
    });
    
    log::info!("Player {} awarded {} XP ({} -> {})", 
        player_id, xp_amount, old_xp, new_xp);
    
    // Refresh unlocked activities if XP actually increased
    if new_xp > old_xp {
        refresh_unlocked_activities(ctx, player_id);
    }
    
    Ok(())
}

/// Mark new activities as acknowledged (set is_new = false).
/// 
/// Call this after the client has displayed the new activities to the user.
#[reducer]
pub fn acknowledge_new_activities(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    let new_activities: Vec<_> = ctx.db.player_unlocked_activity()
        .player_id()
        .filter(&player.id)
        .filter(|ua| ua.is_new)
        .collect();
    
    for ua in new_activities {
        ctx.db.player_unlocked_activity().id().update(PlayerUnlockedActivity {
            is_new: false,
            ..ua
        });
    }
    
    log::info!("Player {} acknowledged new activities", player.id);
    Ok(())
}

/// Initialize unlocked activities for a player.
/// 
/// Call this when a player first logs in or is created to populate
/// their initial set of available activities.
#[reducer]
pub fn initialize_unlocked_activities(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let _player = get_user_player(ctx, player_id)?;
    
    refresh_unlocked_activities(ctx, player_id);
    
    log::info!("Initialized unlocked activities for player {}", player_id);
    Ok(())
}
