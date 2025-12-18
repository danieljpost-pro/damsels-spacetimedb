# Seed Data

Fake activity and equipment data for development and testing.

## Prerequisites

- SpacetimeDB module built with `--features dev`
- Registered player identity (reducers require authentication)

## Files

| File | Purpose |
|------|---------|
| `activities_and_equipment.json` | Master data file |
| `seed_activities.sh` | CLI seeder script |
| `seed_activities.js` | Browser/Node.js seeder |

## Usage

### Option 1: Bash Script (CLI)

```bash
# Requires: jq, spacetime CLI
./seed_activities.sh <MODULE_NAME>

# Example
./seed_activities.sh damsels-dev
```

### Option 2: Browser Console

```javascript
// After connecting to SpacetimeDB
await window.seedActivities.seedAll(client);
```

### Option 3: Clear & Reseed

```javascript
// Clear existing data first
await window.seedActivities.clearAllActivityData(client);
await window.seedActivities.seedAll(client);
```

## Data Summary

- **6** Categories
- **15** Equipment items
- **20** Activities (Skills + Activities)
- **16** Prerequisite relationships
- **22** Activity-Equipment links

