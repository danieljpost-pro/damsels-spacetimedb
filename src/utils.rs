//! Utility functions for the Damsels SpacetimeDB module.

use spacetimedb::ReducerContext;

/// Generate a 5-character room code.
/// Uses alphanumeric characters excluding ambiguous ones (0, O, I, L, 1).
pub fn generate_room_code(ctx: &ReducerContext) -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
    let micros = ctx.timestamp.to_duration_since_unix_epoch().unwrap_or_default().as_micros();
    
    // Generate 5 characters from timestamp entropy
    let mut code = String::with_capacity(5);
    let mut n = micros;
    for _ in 0..5 {
        let idx = (n % CHARS.len() as u128) as usize;
        code.push(CHARS[idx] as char);
        n /= CHARS.len() as u128;
    }
    code
}

/// Generate a unique invitation token.
/// Uses timestamp and identity hash for uniqueness.
pub fn generate_invitation_token(ctx: &ReducerContext) -> String {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    const CHARS: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let micros = ctx.timestamp.to_duration_since_unix_epoch().unwrap_or_default().as_micros();
    
    // Hash the identity for additional entropy
    let mut hasher = DefaultHasher::new();
    ctx.sender.hash(&mut hasher);
    let identity_hash = hasher.finish() as u128;
    
    // Combine timestamp and identity hash for unique token
    let combined = micros.wrapping_add(identity_hash);
    
    // Generate 16 character token
    let mut token = String::with_capacity(16);
    let mut n = combined;
    for _ in 0..16 {
        let idx = (n % CHARS.len() as u128) as usize;
        token.push(CHARS[idx] as char);
        n /= CHARS.len() as u128;
    }
    token
}

