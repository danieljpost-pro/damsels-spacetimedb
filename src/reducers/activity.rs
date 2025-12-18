//! Activity-related reducers for players.
//!
//! Players can update PlayerActivity records for themselves or other players
//! in the same room.

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::player::player;
use crate::models::room::room_member;
use crate::models::player_activity::{player_activity, prerequisite_vouch, PlayerActivity, PrerequisiteVouch};
use crate::models::activity::activity;
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
    });
    
    log::info!("Player {} started activity {}", player.id, activity_id);
    Ok(())
}

/// Mark an activity as complete for a player.
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
    
    // Update to completed
    ctx.db.player_activity().id().update(PlayerActivity {
        status: ActivityStatus::Completed,
        completed_at: Some(ctx.timestamp),
        completed_by: Some(player.id),
        ..pa
    });
    
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
