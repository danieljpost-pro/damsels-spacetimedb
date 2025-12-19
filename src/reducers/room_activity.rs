//! Room Activity reducers for activity selection and completion flow.
//!
//! This module handles the lifecycle of activities within a room:
//! 1. Select/View an activity (Viewing state)
//! 2. Start the activity (InProgress state, records participants)
//! 3. Complete the activity (awards XP to all participants)
//! 4. Cancel at any point (returns to lobby)

use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::activity::activity;
use crate::models::player::{player, Player};
use crate::models::room::room_member;
use crate::models::room_activity::{room_activity, activity_participant, RoomActivity, ActivityParticipant};
use crate::models::player_activity::{player_activity, player_unlocked_activity, player_not_wanted_activity, PlayerActivity, PlayerNotWantedActivity};
use crate::models::enums::{ActivityStatus, RoomActivityStatus};
use crate::reducers::auth::require_user;
use crate::reducers::activity::refresh_unlocked_activities;
use crate::reducers::preferences::get_player_effective_categories;

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

/// Helper: Verify player is a member of the room.
fn verify_room_membership(ctx: &ReducerContext, player_id: u64, room_id: u64) -> Result<(), String> {
    let is_member = ctx.db.room_member()
        .room_id()
        .filter(&room_id)
        .any(|m| m.player_id == player_id);
    
    if !is_member {
        return Err("You are not a member of this room".to_string());
    }
    
    Ok(())
}

/// Helper: Get the current active room activity (Viewing or InProgress).
fn get_active_room_activity(ctx: &ReducerContext, room_id: u64) -> Option<RoomActivity> {
    ctx.db.room_activity()
        .room_id()
        .filter(&room_id)
        .find(|ra| ra.status == RoomActivityStatus::Viewing || ra.status == RoomActivityStatus::InProgress)
}

/// Helper: Check if player has this activity unlocked.
fn is_activity_unlocked(ctx: &ReducerContext, player_id: u64, activity_id: u64) -> bool {
    // Check player_unlocked_activity table
    ctx.db.player_unlocked_activity()
        .player_id()
        .filter(&player_id)
        .any(|ua| ua.activity_id == activity_id)
}

/// Helper: Check if any room member has marked this activity as "not wanted".
fn is_activity_not_wanted_by_room(ctx: &ReducerContext, room_id: u64, activity_id: u64) -> bool {
    // Get all room member IDs
    let room_member_ids: Vec<u64> = ctx.db.room_member()
        .room_id()
        .filter(&room_id)
        .map(|m| m.player_id)
        .collect();
    
    // Check if any member has marked this activity as not wanted
    for member_id in room_member_ids {
        if ctx.db.player_not_wanted_activity()
            .player_id()
            .filter(&member_id)
            .any(|nw| nw.activity_id == activity_id)
        {
            return true;
        }
    }
    
    false
}

/// Helper: Check if an activity's category is in all room members' preferred categories.
/// An activity is only allowed if ALL room members have selected its category.
fn is_activity_allowed_by_room_preferences(ctx: &ReducerContext, room_id: u64, activity_id: u64) -> bool {
    // Get the activity's category
    let activity = match ctx.db.activity().id().find(&activity_id) {
        Some(a) => a,
        None => return false,
    };
    let category_id = activity.category_id;
    
    // Get all room member IDs
    let room_member_ids: Vec<u64> = ctx.db.room_member()
        .room_id()
        .filter(&room_id)
        .map(|m| m.player_id)
        .collect();
    
    // Check that ALL members have this category in their preferences
    for member_id in room_member_ids {
        let member_categories = get_player_effective_categories(ctx, member_id);
        if !member_categories.contains(&category_id) {
            return false;
        }
    }
    
    true
}

/// Get the intersection of all room members' category preferences.
/// Returns the set of category IDs that ALL members have selected.
fn get_room_allowed_categories(ctx: &ReducerContext, room_id: u64) -> Vec<u64> {
    let room_member_ids: Vec<u64> = ctx.db.room_member()
        .room_id()
        .filter(&room_id)
        .map(|m| m.player_id)
        .collect();
    
    if room_member_ids.is_empty() {
        return vec![];
    }
    
    // Start with the first member's categories
    let mut allowed: Vec<u64> = get_player_effective_categories(ctx, room_member_ids[0]);
    
    // Intersect with each other member's categories
    for member_id in room_member_ids.iter().skip(1) {
        let member_cats = get_player_effective_categories(ctx, *member_id);
        allowed.retain(|cat_id| member_cats.contains(cat_id));
    }
    
    allowed
}

/// Select an activity to view in detail.
///
/// Creates a RoomActivity in "Viewing" state. All room members will see
/// this activity displayed in full detail.
///
/// # Arguments
/// * `player_id` - The player selecting the activity
/// * `room_id` - The room where the activity will be viewed
/// * `activity_id` - The activity to view
///
/// # Errors
/// - Player not in room
/// - Activity not found
/// - Activity not unlocked for the selecting player
/// - Another activity already in progress
#[reducer]
pub fn select_room_activity(
    ctx: &ReducerContext,
    player_id: u64,
    room_id: u64,
    activity_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    verify_room_membership(ctx, player.id, room_id)?;
    
    // Verify activity exists
    let act = ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    
    // Verify activity is unlocked for this player
    if !is_activity_unlocked(ctx, player.id, activity_id) {
        return Err("Activity is not unlocked for you".to_string());
    }
    
    // Check if any room member has marked this activity as not wanted
    if is_activity_not_wanted_by_room(ctx, room_id, activity_id) {
        return Err("This activity has been marked as 'not wanted' by a room member".to_string());
    }
    
    // Check if activity's category is allowed by all room members' preferences
    if !is_activity_allowed_by_room_preferences(ctx, room_id, activity_id) {
        return Err("This activity's category is not in all room members' preferences".to_string());
    }
    
    // Check for existing active activity
    if let Some(existing) = get_active_room_activity(ctx, room_id) {
        if existing.status == RoomActivityStatus::InProgress {
            return Err("Another activity is already in progress".to_string());
        }
        // Delete existing Viewing activity (no record preserved)
        ctx.db.room_activity().id().delete(&existing.id);
    }
    
    // Create new room activity in Viewing state
    ctx.db.room_activity().insert(RoomActivity {
        id: 0,
        room_id,
        activity_id,
        status: RoomActivityStatus::Viewing,
        started_by: player.id,
        created_at: ctx.timestamp,
        started_at: None,
        completed_at: None,
    });
    
    log::info!("Player {} selected activity '{}' for viewing in room {}", 
        player.id, act.name, room_id);
    Ok(())
}

/// Randomly select an activity from the player's unlocked activities.
///
/// Uses weighted random selection based on ratings from all room members.
/// Activities rated highly by room members are more likely to be selected.
/// Unrated activities use a default weight of 3 (middle of 1-5 scale).
///
/// # Algorithm
/// 1. Get all unlocked activities for the triggering player
/// 2. For each activity, calculate average rating from all room members
/// 3. Use weighted random selection where weight = average rating
///
/// # Arguments
/// * `player_id` - The player triggering the random selection
/// * `room_id` - The room where the activity will be viewed
///
/// # Errors
/// - Player not in room
/// - No unlocked activities available
/// - Another activity already in progress
#[reducer]
pub fn random_room_activity(
    ctx: &ReducerContext,
    player_id: u64,
    room_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    verify_room_membership(ctx, player.id, room_id)?;
    
    // Check for existing in-progress activity
    if let Some(existing) = get_active_room_activity(ctx, room_id) {
        if existing.status == RoomActivityStatus::InProgress {
            return Err("Another activity is already in progress".to_string());
        }
        // Delete existing Viewing activity (no record preserved)
        ctx.db.room_activity().id().delete(&existing.id);
    }
    
    // Get all unlocked activities for this player
    let all_unlocked: Vec<_> = ctx.db.player_unlocked_activity()
        .player_id()
        .filter(&player.id)
        .collect();
    
    // Get the intersection of all room members' category preferences
    let allowed_categories = get_room_allowed_categories(ctx, room_id);
    
    // Filter out activities that:
    // 1. Any room member has marked as "not wanted"
    // 2. Are not in all room members' category preferences
    let unlocked: Vec<_> = all_unlocked
        .into_iter()
        .filter(|ua| {
            // Check not wanted
            if is_activity_not_wanted_by_room(ctx, room_id, ua.activity_id) {
                return false;
            }
            // Check category preference
            if let Some(act) = ctx.db.activity().id().find(&ua.activity_id) {
                allowed_categories.contains(&act.category_id)
            } else {
                false
            }
        })
        .collect();
    
    if unlocked.is_empty() {
        return Err("No available activities (all may be filtered by preferences or marked as 'not wanted')".to_string());
    }
    
    // Get all room member IDs for rating lookup
    let room_member_ids: Vec<u64> = ctx.db.room_member()
        .room_id()
        .filter(&room_id)
        .map(|m| m.player_id)
        .collect();
    
    // Calculate weights for each activity based on room members' ratings
    let mut weighted_activities: Vec<(usize, u64)> = Vec::new(); // (index, weight * 100)
    let mut total_weight: u64 = 0;
    
    for (idx, ua) in unlocked.iter().enumerate() {
        // Collect ratings from room members for this activity
        let mut rating_sum: u64 = 0;
        let mut rating_count: u64 = 0;
        
        for member_id in &room_member_ids {
            if let Some(pa) = ctx.db.player_activity()
                .player_id()
                .filter(member_id)
                .find(|pa| pa.activity_id == ua.activity_id)
            {
                if let Some(rating) = pa.rating {
                    rating_sum += rating as u64;
                    rating_count += 1;
                }
            }
        }
        
        // Calculate weight: use average rating * 100, or default of 300 (rating 3)
        let weight = if rating_count > 0 {
            (rating_sum * 100) / rating_count
        } else {
            300 // Default weight for unrated activities (equivalent to rating 3)
        };
        
        weighted_activities.push((idx, weight));
        total_weight += weight;
    }
    
    // Weighted random selection
    let random_point = (ctx.timestamp.to_micros_since_unix_epoch().unsigned_abs() % total_weight) as u64;
    let mut cumulative: u64 = 0;
    let mut selected_idx = 0;
    
    for (idx, weight) in &weighted_activities {
        cumulative += weight;
        if random_point < cumulative {
            selected_idx = *idx;
            break;
        }
    }
    
    let selected = &unlocked[selected_idx];
    
    // Create new room activity in Viewing state
    ctx.db.room_activity().insert(RoomActivity {
        id: 0,
        room_id,
        activity_id: selected.activity_id,
        status: RoomActivityStatus::Viewing,
        started_by: player.id,
        created_at: ctx.timestamp,
        started_at: None,
        completed_at: None,
    });
    
    log::info!("Player {} randomly selected activity '{}' for room {} (weighted by ratings)", 
        player.id, selected.activity_name, room_id);
    Ok(())
}

/// Start the currently viewed activity.
///
/// Moves the room activity from "Viewing" to "InProgress" state.
/// Records all current room members as participants with their roles.
///
/// # Arguments
/// * `player_id` - The player starting the activity
/// * `room_id` - The room where the activity is being started
///
/// # Errors
/// - Player not in room
/// - No activity currently being viewed
/// - Activity already in progress
#[reducer]
pub fn start_room_activity(
    ctx: &ReducerContext,
    player_id: u64,
    room_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    verify_room_membership(ctx, player.id, room_id)?;
    
    // Get current viewing activity
    let room_act = get_active_room_activity(ctx, room_id)
        .ok_or("No activity is currently being viewed")?;
    
    if room_act.status != RoomActivityStatus::Viewing {
        return Err("Activity is already in progress".to_string());
    }
    
    // Update to InProgress
    ctx.db.room_activity().id().update(RoomActivity {
        status: RoomActivityStatus::InProgress,
        started_at: Some(ctx.timestamp),
        ..room_act.clone()
    });
    
    // Record all room members as participants with their current roles
    for member in ctx.db.room_member().room_id().filter(&room_id) {
        ctx.db.activity_participant().insert(ActivityParticipant {
            id: 0,
            room_activity_id: room_act.id,
            player_id: member.player_id,
            role: member.role,
            xp_earned: 0,
            completed: false,
        });
    }
    
    log::info!("Player {} started activity {} in room {}", 
        player.id, room_act.activity_id, room_id);
    Ok(())
}

/// Complete the current room activity.
///
/// Awards XP to all participants and records the activity as completed
/// for each player. Updates player XP and refreshes unlocked activities.
///
/// # Arguments
/// * `player_id` - The player marking the activity as complete
/// * `room_id` - The room where the activity was performed
///
/// # Errors
/// - Player not in room
/// - No activity currently in progress
#[reducer]
pub fn complete_room_activity(
    ctx: &ReducerContext,
    player_id: u64,
    room_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    verify_room_membership(ctx, player.id, room_id)?;
    
    // Get current in-progress activity
    let room_act = get_active_room_activity(ctx, room_id)
        .ok_or("No activity is currently in progress")?;
    
    if room_act.status != RoomActivityStatus::InProgress {
        return Err("Activity is not in progress".to_string());
    }
    
    // Get the activity to find XP reward
    let act = ctx.db.activity().id().find(&room_act.activity_id)
        .ok_or("Activity not found")?;
    
    let xp_reward = act.xp_reward;
    
    // Update room activity to Completed
    ctx.db.room_activity().id().update(RoomActivity {
        status: RoomActivityStatus::Completed,
        completed_at: Some(ctx.timestamp),
        ..room_act.clone()
    });
    
    // Award XP to all participants
    let participants: Vec<_> = ctx.db.activity_participant()
        .room_activity_id()
        .filter(&room_act.id)
        .collect();
    
    let participant_count = participants.len();
    
    for participant in participants {
        // Update participant record with XP earned
        ctx.db.activity_participant().id().update(ActivityParticipant {
            xp_earned: xp_reward,
            completed: true,
            ..participant.clone()
        });
        
        // Award XP to player
        if let Some(p) = ctx.db.player().id().find(&participant.player_id) {
            let new_xp = p.xp.saturating_add(xp_reward);
            ctx.db.player().id().update(Player {
                xp: new_xp,
                ..p.clone()
            });
            
            log::info!("Player {} earned {} XP (now has {} XP)", 
                participant.player_id, xp_reward, new_xp);
        }
        
        // Record in player_activity if not already tracked
        let existing_pa = ctx.db.player_activity()
            .player_id()
            .filter(&participant.player_id)
            .find(|pa| pa.activity_id == room_act.activity_id);
        
        if let Some(pa) = existing_pa {
            // Update to completed
            ctx.db.player_activity().id().update(PlayerActivity {
                status: ActivityStatus::Completed,
                completed_at: Some(ctx.timestamp),
                completed_by: Some(player.id),
                ..pa
            });
        } else {
            // Create new completed record
            ctx.db.player_activity().insert(PlayerActivity {
                id: 0,
                player_id: participant.player_id,
                activity_id: room_act.activity_id,
                status: ActivityStatus::Completed,
                completed_at: Some(ctx.timestamp),
                completed_by: Some(player.id),
                vouched: false,
                rating: None, // Player can rate after completion
            });
        }
        
        // Refresh unlocked activities for this player
        refresh_unlocked_activities(ctx, participant.player_id);
    }
    
    log::info!("Activity {} completed in room {} by player {}. {} participants awarded {} XP each.", 
        room_act.activity_id, room_id, player.id, participant_count, xp_reward);
    Ok(())
}

/// Cancel the current room activity.
///
/// Deletes the room activity and any participant records. No record is
/// preserved of cancelled activities - it's as if they never happened.
/// No XP is awarded.
///
/// # Arguments
/// * `player_id` - The player cancelling the activity
/// * `room_id` - The room where the activity is being cancelled
///
/// # Errors
/// - Player not in room
/// - No active activity to cancel
#[reducer]
pub fn cancel_room_activity(
    ctx: &ReducerContext,
    player_id: u64,
    room_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    verify_room_membership(ctx, player.id, room_id)?;
    
    // Get current active activity
    let room_act = get_active_room_activity(ctx, room_id)
        .ok_or("No active activity to cancel")?;
    
    // Delete any participant records (if activity was InProgress)
    let participants: Vec<_> = ctx.db.activity_participant()
        .room_activity_id()
        .filter(&room_act.id)
        .collect();
    
    for participant in participants {
        ctx.db.activity_participant().id().delete(&participant.id);
    }
    
    // Delete the room activity record entirely
    ctx.db.room_activity().id().delete(&room_act.id);
    
    log::info!("Player {} cancelled and deleted activity {} in room {}", 
        player.id, room_act.activity_id, room_id);
    Ok(())
}

/// Get the current room activity state.
///
/// Returns the currently active room activity (if any) for subscription updates.
/// This is primarily for debugging; clients should subscribe to room_activity table.
#[reducer]
pub fn get_room_activity_state(
    ctx: &ReducerContext,
    player_id: u64,
    room_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    verify_room_membership(ctx, player.id, room_id)?;
    
    if let Some(room_act) = get_active_room_activity(ctx, room_id) {
        let act = ctx.db.activity().id().find(&room_act.activity_id);
        let act_name = act.map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string());
        
        log::info!("Room {} activity state: {:?} - {} (id: {})", 
            room_id, room_act.status, act_name, room_act.activity_id);
    } else {
        log::info!("Room {} has no active activity", room_id);
    }
    
    Ok(())
}

/// Refresh the available activities for a room.
/// 
/// This triggers a log message with the count of available activities after
/// filtering by category preferences and not-wanted lists. The actual filtering
/// is done client-side by subscribing to the relevant tables.
/// 
/// # Arguments
/// * `player_id` - The player requesting the refresh
/// * `room_id` - The room to check activities for
#[reducer]
pub fn refresh_room_activities(
    ctx: &ReducerContext,
    player_id: u64,
    room_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    verify_room_membership(ctx, player.id, room_id)?;
    
    // Get the intersection of all room members' category preferences
    let allowed_categories = get_room_allowed_categories(ctx, room_id);
    
    // Count available activities
    let all_unlocked: Vec<_> = ctx.db.player_unlocked_activity()
        .player_id()
        .filter(&player.id)
        .collect();
    
    let available_count = all_unlocked
        .iter()
        .filter(|ua| {
            if is_activity_not_wanted_by_room(ctx, room_id, ua.activity_id) {
                return false;
            }
            if let Some(act) = ctx.db.activity().id().find(&ua.activity_id) {
                allowed_categories.contains(&act.category_id)
            } else {
                false
            }
        })
        .count();
    
    log::info!("Room {} has {} available activities for player {} (from {} unlocked, {} allowed categories)", 
        room_id, available_count, player.id, all_unlocked.len(), allowed_categories.len());
    Ok(())
}

/// Mark an activity as "not wanted" by a player.
///
/// Activities marked as not wanted by any room member will not appear in
/// that room's activity list or be selected by "Choose for me".
///
/// # Arguments
/// * `player_id` - The player marking the activity
/// * `activity_id` - The activity to mark as not wanted
///
/// # Errors
/// - Player not found
/// - Activity not found
/// - Already marked as not wanted
#[reducer]
pub fn mark_activity_not_wanted(
    ctx: &ReducerContext,
    player_id: u64,
    activity_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Verify activity exists
    let act = ctx.db.activity().id().find(&activity_id)
        .ok_or("Activity not found")?;
    
    // Check if already marked
    let already_marked = ctx.db.player_not_wanted_activity()
        .player_id()
        .filter(&player.id)
        .any(|nw| nw.activity_id == activity_id);
    
    if already_marked {
        return Err("Activity is already marked as not wanted".to_string());
    }
    
    // Create the not-wanted record
    ctx.db.player_not_wanted_activity().insert(PlayerNotWantedActivity {
        id: 0,
        player_id: player.id,
        activity_id,
        created_at: ctx.timestamp,
    });
    
    log::info!("Player {} marked activity '{}' (id: {}) as not wanted", 
        player.id, act.name, activity_id);
    Ok(())
}

/// Remove an activity from a player's "not wanted" list.
///
/// After calling this, the activity will appear again in rooms where
/// this player is a member (unless another member has it marked).
///
/// # Arguments
/// * `player_id` - The player unmarking the activity
/// * `activity_id` - The activity to remove from not-wanted list
///
/// # Errors
/// - Player not found
/// - Activity not in not-wanted list
#[reducer]
pub fn unmark_activity_not_wanted(
    ctx: &ReducerContext,
    player_id: u64,
    activity_id: u64,
) -> Result<(), String> {
    let player = get_user_player(ctx, player_id)?;
    
    // Find and delete the not-wanted record
    let nw_record = ctx.db.player_not_wanted_activity()
        .player_id()
        .filter(&player.id)
        .find(|nw| nw.activity_id == activity_id)
        .ok_or("Activity is not in your not-wanted list")?;
    
    ctx.db.player_not_wanted_activity().id().delete(&nw_record.id);
    
    log::info!("Player {} removed activity {} from not-wanted list", 
        player.id, activity_id);
    Ok(())
}

