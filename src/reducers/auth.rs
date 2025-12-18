//! Authentication reducers for User registration and login.

use sha2::{Sha256, Digest};
use spacetimedb::{reducer, ReducerContext, Table};

use crate::models::user::{user, User};
use crate::models::player::{player, Player};
use crate::models::room::room_member;
use crate::models::enums::UserRole;

/// Hash a password using SHA-256.
/// DEV MODE: Uses username as salt for simplicity (works across browser sessions).
/// In production, use a proper KDF like argon2 or bcrypt with random salt.
fn hash_password(password: &str, username: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify a password against a stored hash.
fn verify_password(password: &str, username: &str, stored_hash: &str) -> bool {
    hash_password(password, username) == stored_hash
}

/// Register a new user account with username and password.
/// Returns error if username is taken.
#[reducer]
pub fn register_user(ctx: &ReducerContext, username: String, password: String) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Username cannot be empty".to_string());
    }
    if password.len() < 4 {
        return Err("Password must be at least 4 characters".to_string());
    }

    // Check if username is taken
    if ctx.db.user().username().find(&username).is_some() {
        return Err("Username already taken".to_string());
    }

    // Use username as salt (works across browser sessions in dev mode)
    let password_hash = hash_password(&password, &username);

    ctx.db.user().insert(User {
        id: 0,
        identity: ctx.sender,
        username: username.clone(),
        password_hash,
        role: UserRole::Player,
        created_at: ctx.timestamp,
        last_seen: ctx.timestamp,
    });

    log::info!("User registered: {}", username);
    Ok(())
}

/// Login an existing user with username and password.
/// Updates the user's identity to the current session identity.
#[reducer]
pub fn login_user(ctx: &ReducerContext, username: String, password: String) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    let user = match ctx.db.user().username().find(&username) {
        Some(u) => u,
        None => {
            #[cfg(feature = "dev")]
            {
                // In dev mode, show which username was not found
                let all_usernames: Vec<String> = ctx.db.user().iter().map(|u| u.username.clone()).collect();
                return Err(format!(
                    "User '{}' not found. Existing users: {:?}",
                    username,
                    all_usernames
                ));
            }
            #[cfg(not(feature = "dev"))]
            return Err("Invalid username or password".to_string());
        }
    };

    // Verify password using username as salt
    if !verify_password(&password, &username, &user.password_hash) {
        #[cfg(feature = "dev")]
        return Err(format!(
            "Password mismatch for user '{}'. Expected hash: {}, Got hash: {}",
            username,
            user.password_hash,
            hash_password(&password, &username)
        ));
        #[cfg(not(feature = "dev"))]
        return Err("Invalid username or password".to_string());
    }

    // Update identity to current session and last_seen
    ctx.db.user().id().update(User {
        identity: ctx.sender,
        last_seen: ctx.timestamp,
        ..user
    });

    log::info!("User logged in: {}", username);
    Ok(())
}

/// Logout the current user.
/// Updates last_seen timestamp. The client should clear its token after calling this.
#[reducer]
pub fn logout_user(ctx: &ReducerContext) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    // Update last_seen
    ctx.db.user().id().update(User {
        last_seen: ctx.timestamp,
        ..user.clone()
    });
    
    log::info!("User logged out: {}", user.username);
    Ok(())
}

/// Get the currently authenticated user (if any).
/// This is a helper used by other reducers.
pub fn get_authenticated_user(ctx: &ReducerContext) -> Option<User> {
    ctx.db.user().identity().find(&ctx.sender)
}

/// Require an authenticated user, returning error if not logged in.
pub fn require_user(ctx: &ReducerContext) -> Result<User, String> {
    get_authenticated_user(ctx).ok_or_else(|| "Not logged in".to_string())
}

/// Create a new player identity for the authenticated user.
/// A user can have multiple player identities.
#[reducer]
pub fn create_player(ctx: &ReducerContext, player_name: String) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    if player_name.trim().is_empty() {
        return Err("Player name cannot be empty".to_string());
    }

    // Check if player name is taken by anyone
    if ctx.db.player().username().find(&player_name).is_some() {
        return Err("Player name already taken".to_string());
    }

    ctx.db.player().insert(Player {
        id: 0,
        user_id: user.id,
        username: player_name.clone(),
        xp: 0,
        created_at: ctx.timestamp,
    });

    log::info!("Player created: {} for user {}", player_name, user.username);
    Ok(())
}

/// Get all player identities for the authenticated user.
/// Returns a list of player IDs and names via subscription.
/// (Client should subscribe to player table filtered by user_id)

/// Delete a player identity (only if not currently in a room).
#[reducer]
pub fn delete_player(ctx: &ReducerContext, player_id: u64) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("You can only delete your own players".to_string());
    }

    // Check if player is in any room
    let in_room = ctx.db.room_member()
        .player_id()
        .filter(&player_id)
        .count() > 0;
    
    if in_room {
        return Err("Cannot delete player while in a room".to_string());
    }

    ctx.db.player().id().delete(&player_id);
    log::info!("Player {} deleted", player_id);
    Ok(())
}

/// Rename a player identity.
#[reducer]
pub fn rename_player(ctx: &ReducerContext, player_id: u64, new_name: String) -> Result<(), String> {
    let user = require_user(ctx)?;
    
    if new_name.trim().is_empty() {
        return Err("Player name cannot be empty".to_string());
    }
    
    let player = ctx.db.player().id().find(&player_id)
        .ok_or("Player not found")?;
    
    if player.user_id != user.id {
        return Err("You can only rename your own players".to_string());
    }

    // Check if new name is taken by another player
    if let Some(existing) = ctx.db.player().username().find(&new_name) {
        if existing.id != player_id {
            return Err("Player name already taken".to_string());
        }
    }

    ctx.db.player().id().update(Player {
        username: new_name.clone(),
        ..player
    });

    log::info!("Player {} renamed to {}", player_id, new_name);
    Ok(())
}

