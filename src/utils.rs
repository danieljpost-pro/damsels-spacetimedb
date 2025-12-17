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

