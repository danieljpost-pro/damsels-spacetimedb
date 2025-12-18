# Damsels - SpacetimeDB Game Backend

A session-based, real-time multiplayer game backend built with Rust and [SpacetimeDB](https://spacetimedb.com/). Features a User/Player authentication model, room-based sessions, and an Activity Tree progression system.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SpacetimeDB Module                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                           REDUCERS                                   │   │
│  │  (Client-callable functions that modify state)                       │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  auth.rs           player.rs          room.rs         activity.rs   │   │
│  │  ─────────         ──────────         ────────        ────────────  │   │
│  │  • register_user   • create_room      • revoke_       • initialize_ │   │
│  │  • login_user      • join_room          invitation      unlocked_   │   │
│  │  • logout_user     • leave_room                         activities  │   │
│  │  • create_player   • change_role                      • acknowledge │   │
│  │  • delete_player   • create_room_                       _new_       │   │
│  │  • rename_player     invitation                         activities  │   │
│  │                    • close_room                       • award_xp    │   │
│  │                    • accept_                                        │   │
│  │                      invitation                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                            MODELS                                    │   │
│  │  (Database tables with automatic subscription support)               │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │   │
│  │  │    User      │  │    Player    │  │         Room             │  │   │
│  │  │ ──────────── │  │ ──────────── │  │ ──────────────────────── │  │   │
│  │  │ • identity   │◀─│ • user_id    │  │ • code (5-char)          │  │   │
│  │  │ • username   │  │ • username   │─▶│ • owner_id               │  │   │
│  │  │ • password   │  │ • xp         │  │ • is_open                │  │   │
│  │  │   _hash      │  │              │  │                          │  │   │
│  │  │ • role       │  │              │  │ RoomMember               │  │   │
│  │  └──────────────┘  └──────────────┘  │ • player_id, role        │  │   │
│  │                                      │                          │  │   │
│  │                                      │ RoomInvitation           │  │   │
│  │                                      │ • token, status          │  │   │
│  │                                      └──────────────────────────┘  │   │
│  │                                                                      │   │
│  │  ┌──────────────────────────────────────────────────────────────┐   │   │
│  │  │                    Activity System                            │   │   │
│  │  │ ────────────────────────────────────────────────────────────  │   │   │
│  │  │                                                               │   │   │
│  │  │  ActivityCategory ──▶ Activity ──▶ PlayerActivity            │   │   │
│  │  │  (e.g. "Bondage")    (e.g. "Basic   (player's progress)      │   │   │
│  │  │                       Rope Work")                             │   │   │
│  │  │                           │                                   │   │   │
│  │  │                           ├── ActivityPrerequisite            │   │   │
│  │  │                           │   (skill dependencies)            │   │   │
│  │  │                           │                                   │   │   │
│  │  │                           └── ActivityEquipment               │   │   │
│  │  │                               (required items)                │   │   │
│  │  │                                                               │   │   │
│  │  │  PlayerUnlockedActivity ─── Denormalized view of available   │   │   │
│  │  │                             activities for efficient queries  │   │   │
│  │  └──────────────────────────────────────────────────────────────┘   │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Data Model

### Authentication Model

```
User (Account)                  Player (Identity)
┌─────────────────┐            ┌─────────────────┐
│ id: u64         │            │ id: u64         │
│ identity: bytes │◀───────────│ user_id: u64    │
│ username: str   │   1:many   │ username: str   │
│ password_hash   │            │ xp: u32         │
│ role: UserRole  │            │ created_at      │
│ last_seen       │            └─────────────────┘
└─────────────────┘
```

- A **User** is an authenticated account (login credentials)
- A **Player** is a game identity belonging to a User
- One User can have multiple Player identities
- Each Player has independent XP and activity progress

### Room Model

```
Room                           RoomMember                    RoomInvitation
┌─────────────────┐           ┌─────────────────┐           ┌─────────────────┐
│ id: u64         │           │ id: u64         │           │ id: u64         │
│ code: str (5)   │◀──────────│ room_id: u64    │           │ room_id: u64    │
│ name: str       │           │ player_id: u64  │           │ token: str (5)  │
│ owner_id: u64   │           │ role: PlayerRole│           │ created_by: u64 │
│ is_open: bool   │           │ joined_at       │           │ for_username?   │
│ created_at      │           └─────────────────┘           │ status          │
└─────────────────┘                                         │ accepted_by?    │
                                                            └─────────────────┘

PlayerRole = Top | Bottom | Observer | Photographer | ActivityAdmin
```

### Activity Progression System

```
ActivityCategory              Activity                      PlayerActivity
┌─────────────────┐          ┌─────────────────┐           ┌─────────────────┐
│ id: u64         │          │ id: u64         │           │ id: u64         │
│ name: str       │◀─────────│ category_id     │◀──────────│ activity_id     │
│ description     │          │ kind: Skill|Act │           │ player_id       │
│ display_order   │          │ name: str       │           │ status          │
└─────────────────┘          │ description     │           │ completed_at?   │
                             │ instructions    │           │ completed_by?   │
                             │ video_url?      │           │ vouched: bool   │
                             │ xp_required     │           └─────────────────┘
                             │ xp_reward       │
                             └─────────────────┘
                                    │
                                    │
        ┌───────────────────────────┴────────────────────────┐
        │                                                    │
ActivityPrerequisite                              ActivityEquipment
┌─────────────────┐                              ┌─────────────────┐
│ activity_id     │                              │ activity_id     │
│ prerequisite_id │ (another Activity)           │ equipment_id    │
│ vouch_count     │                              │ notes?          │
└─────────────────┘                              └─────────────────┘
```

## Reducer API Reference

### Authentication (`auth.rs`)

| Reducer | Arguments | Description |
|---------|-----------|-------------|
| `register_user` | `username, password` | Create new user account |
| `login_user` | `username, password` | Authenticate and update identity |
| `logout_user` | - | Update last_seen timestamp |
| `create_player` | `player_name` | Create new player identity |
| `delete_player` | `player_id` | Delete player (if not in room) |
| `rename_player` | `player_id, new_name` | Change player display name |

### Room Operations (`player.rs`)

| Reducer | Arguments | Description |
|---------|-----------|-------------|
| `create_room` | `player_id, room_name, role` | Create room and join as owner |
| `join_room` | `player_id, room_code, role` | Join existing room by code |
| `accept_invitation` | `player_id, token, role` | Join room via invitation |
| `leave_room` | `player_id, room_id` | Leave a specific room |
| `change_role` | `player_id, room_id, role` | Change role in room |
| `create_room_invitation` | `player_id, room_id, for_username?` | Create invitation token |
| `close_room` | `player_id, room_id` | Close room (owner only) |

### Room Management (`room.rs`)

| Reducer | Arguments | Description |
|---------|-----------|-------------|
| `revoke_room_invitation` | `player_id, invitation_id` | Revoke active invitation |

### Activity System (`activity.rs`)

| Reducer | Arguments | Description |
|---------|-----------|-------------|
| `initialize_unlocked_activities` | `player_id` | Populate unlocked activities table |
| `acknowledge_new_activities` | `player_id` | Mark new activities as seen |
| `award_xp` | `player_id, xp_amount` | Grant XP and unlock new activities |

### Admin Reducers (Dev Mode Only)

Requires `--features dev` when building:

| Reducer | Description |
|---------|-------------|
| `admin_create_category` | Create activity category |
| `admin_create_activity` | Create activity or skill |
| `admin_add_prerequisite` | Add skill dependency |
| `admin_create_equipment` | Create equipment item |
| `admin_add_activity_equipment` | Link equipment to activity |

## Project Structure

```
damsels-spacetimedb/
├── Cargo.toml              # Dependencies and feature flags
├── seed_data/
│   ├── activities_and_equipment.json  # Sample data
│   ├── seed_activities.js             # Seeding script
│   └── seed_activities.sh             # Shell wrapper
└── src/
    ├── lib.rs              # Module entry point
    ├── admin.rs            # Dev-only admin reducers
    ├── utils.rs            # Helper functions
    ├── models/
    │   ├── mod.rs          # Model exports
    │   ├── enums.rs        # Enum definitions
    │   ├── user.rs         # User table
    │   ├── player.rs       # Player table
    │   ├── room.rs         # Room, RoomMember, RoomInvitation
    │   ├── activity.rs     # Activity, Category, Prerequisite
    │   ├── equipment.rs    # Equipment, ActivityEquipment
    │   ├── player_activity.rs  # PlayerActivity, PlayerUnlockedActivity
    │   ├── player_history.rs   # Historical records
    │   └── room_activity.rs    # RoomActivity, ActivityParticipant
    └── reducers/
        ├── mod.rs          # Reducer exports
        ├── auth.rs         # User/Player authentication
        ├── player.rs       # Room operations
        ├── room.rs         # Additional room management
        ├── activity.rs     # Activity system
        └── lifecycle.rs    # Connection lifecycle
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [SpacetimeDB CLI](https://spacetimedb.com/install)

## Quick Start

### Install SpacetimeDB CLI

```bash
curl -sSf https://install.spacetimedb.com | sh
```

### Start Local SpacetimeDB Server

```bash
spacetime start
```

### Build the Module

```bash
# Development build (includes admin reducers)
spacetime build --features dev

# Production build
spacetime build
```

### Publish the Module

```bash
spacetime publish --server local damsels
```

### Seed Sample Data

```bash
cd seed_data
./seed_activities.sh
```

### Test Reducers

```bash
# Register a user
spacetime call --server local damsels register_user '["testuser", "password123"]'

# Check logs
spacetime logs --server local damsels
```

## Client Subscriptions

Clients can subscribe to these tables for real-time updates:

```sql
SELECT * FROM user
SELECT * FROM player
SELECT * FROM room
SELECT * FROM room_member
SELECT * FROM room_invitation
SELECT * FROM category
SELECT * FROM activity
SELECT * FROM player_activity
SELECT * FROM player_unlocked_activity
```

## Security Notes

- **Password Hashing**: Uses SHA-256 with username as salt (dev mode simplification)
- **For Production**: Replace with Argon2 or bcrypt with random salt
- **Identity Binding**: User identity is tied to SpacetimeDB connection identity
- **Authorization**: All reducers validate player ownership before operations

## Related Layers

- **[damsels-assets](../damsels-assets/)** — Frontend UI (Zola + JavaScript)
- **[damsels-pingora](../damsels-pingora/)** — Reverse proxy (Pingora)
