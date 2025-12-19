#!/bin/bash
# Seed Users and Players from JSON file
# Usage: ./seed_users.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JSON_FILE="${SCRIPT_DIR}/users_and_players.json"
MODULE="damsels"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required but not installed.${NC}"
    echo "Install with: sudo apt install jq"
    exit 1
fi

# Check if JSON file exists
if [ ! -f "$JSON_FILE" ]; then
    echo -e "${RED}Error: $JSON_FILE not found${NC}"
    exit 1
fi

echo -e "${GREEN}=== Seeding Users and Players ===${NC}"
echo "Reading from: $JSON_FILE"
echo ""

# Hash password using SHA-256 with username as salt (matches auth.rs hash_password function)
# Usage: hash_password "username" "password"
hash_password() {
    local username="$1"
    local password="$2"
    echo -n "${username}${password}" | sha256sum | awk '{print $1}'
}

# Count totals
USER_COUNT=$(jq '.users | length' "$JSON_FILE")
PLAYER_COUNT=$(jq '.players | length' "$JSON_FILE")

echo "Found $USER_COUNT users and $PLAYER_COUNT players to seed"
echo ""

# =============================================================================
# Seed Users
# =============================================================================
echo -e "${YELLOW}[1/4] Seeding Users...${NC}"

user_success=0
for i in $(seq 0 $((USER_COUNT - 1))); do
    id=$(jq -r ".users[$i].id" "$JSON_FILE")
    username=$(jq -r ".users[$i].username" "$JSON_FILE")
    password=$(jq -r ".users[$i].password" "$JSON_FILE")
    role=$(jq -r ".users[$i].role" "$JSON_FILE")
    
    # Compute SHA-256 hash with username as salt (matches auth.rs)
    password_hash=$(hash_password "$username" "$password")
    
    echo -e "  ${YELLOW}→${NC} seed_user: $username (id: $id, role: $role)"
    
    # Call the reducer with the password hash
    result=$(spacetime call "$MODULE" seed_user "$id" "\"$username\"" "\"$password_hash\"" "{\"$role\":{}}" 2>&1) || true
    
    if echo "$result" | grep -q "Error"; then
        echo -e "  ${RED}✗ Failed: $result${NC}"
    else
        ((user_success++)) || true
    fi
done

echo -e "${GREEN}✓ $user_success users created${NC}"
echo ""

# =============================================================================
# Seed User Category Preferences
# =============================================================================
echo -e "${YELLOW}[2/4] Seeding User Category Preferences...${NC}"

user_pref_count=0
for i in $(seq 0 $((USER_COUNT - 1))); do
    user_id=$(jq -r ".users[$i].id" "$JSON_FILE")
    username=$(jq -r ".users[$i].username" "$JSON_FILE")
    pref_count=$(jq ".users[$i].category_preferences | length" "$JSON_FILE")
    
    for j in $(seq 0 $((pref_count - 1))); do
        cat_id=$(jq -r ".users[$i].category_preferences[$j]" "$JSON_FILE")
        
        result=$(spacetime call "$MODULE" seed_user_category_preference "$user_id" "$cat_id" 2>&1) || true
        
        if echo "$result" | grep -q "Error"; then
            echo -e "  ${RED}✗ User $user_id → Category $cat_id: $result${NC}"
        else
            ((user_pref_count++)) || true
        fi
    done
    echo -e "  ${YELLOW}→${NC} $username: $pref_count preferences"
done

echo -e "${GREEN}✓ $user_pref_count user preferences created${NC}"
echo ""

# =============================================================================
# Seed Players
# =============================================================================
echo -e "${YELLOW}[3/4] Seeding Players...${NC}"

player_success=0
for i in $(seq 0 $((PLAYER_COUNT - 1))); do
    id=$(jq -r ".players[$i].id" "$JSON_FILE")
    user_id=$(jq -r ".players[$i].user_id" "$JSON_FILE")
    username=$(jq -r ".players[$i].username" "$JSON_FILE")
    xp=$(jq -r ".players[$i].xp" "$JSON_FILE")
    
    echo -e "  ${YELLOW}→${NC} seed_player: $username (id: $id, user_id: $user_id, xp: $xp)"
    
    result=$(spacetime call "$MODULE" seed_player "$id" "$user_id" "\"$username\"" "$xp" 2>&1) || true
    
    if echo "$result" | grep -q "Error"; then
        echo -e "  ${RED}✗ Failed: $result${NC}"
    else
        ((player_success++)) || true
    fi
done

echo -e "${GREEN}✓ $player_success players created${NC}"
echo ""

# =============================================================================
# Seed Player Category Preferences
# =============================================================================
echo -e "${YELLOW}[4/4] Seeding Player Category Preferences...${NC}"

player_pref_count=0
for i in $(seq 0 $((PLAYER_COUNT - 1))); do
    player_id=$(jq -r ".players[$i].id" "$JSON_FILE")
    username=$(jq -r ".players[$i].username" "$JSON_FILE")
    pref_count=$(jq ".players[$i].category_preferences | length" "$JSON_FILE")
    
    for j in $(seq 0 $((pref_count - 1))); do
        cat_id=$(jq -r ".players[$i].category_preferences[$j]" "$JSON_FILE")
        
        result=$(spacetime call "$MODULE" seed_player_category_preference "$player_id" "$cat_id" 2>&1) || true
        
        if echo "$result" | grep -q "Error"; then
            echo -e "  ${RED}✗ Player $player_id → Category $cat_id: $result${NC}"
        else
            ((player_pref_count++)) || true
        fi
    done
    echo -e "  ${YELLOW}→${NC} $username: $pref_count preferences"
done

echo -e "${GREEN}✓ $player_pref_count player preferences created${NC}"
echo ""

# =============================================================================
# Summary
# =============================================================================
echo -e "${GREEN}=== Seeding Complete ===${NC}"
echo "Summary:"
echo "  - Users: $user_success"
echo "  - User Preferences: $user_pref_count"
echo "  - Players: $player_success"
echo "  - Player Preferences: $player_pref_count"
echo ""
echo -e "${GREEN}All user/player data has been seeded successfully!${NC}"
