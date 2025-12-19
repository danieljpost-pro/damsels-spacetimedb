#!/bin/bash
#
# Seed Activities and Equipment Data into SpacetimeDB
#
# This script reads the JSON seed data and calls SpacetimeDB reducers
# to populate the database with activities and equipment.
#
# Prerequisites:
# - jq must be installed
# - spacetime CLI must be installed and configured
# - Module must be built with --features dev
#
# Usage:
#   ./seed_activities.sh [MODULE_NAME]
#
# Example:
#   ./seed_activities.sh damsels-dev
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SEED_FILE="${SCRIPT_DIR}/activities_and_equipment.json"
MODULE_NAME="${1:-damsels}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== SpacetimeDB Activity & Equipment Seeder ===${NC}"
echo "Module: ${MODULE_NAME}"
echo "Seed file: ${SEED_FILE}"
echo ""

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required but not installed.${NC}"
    echo "Install with: sudo apt install jq"
    exit 1
fi

if ! command -v spacetime &> /dev/null; then
    echo -e "${RED}Error: spacetime CLI is required but not installed.${NC}"
    exit 1
fi

if [ ! -f "${SEED_FILE}" ]; then
    echo -e "${RED}Error: Seed file not found: ${SEED_FILE}${NC}"
    exit 1
fi

# Helper function to format Optional strings for SpacetimeDB CLI
# SpacetimeDB expects: {"some": "value"} or {"none": []}
format_option() {
    local value="$1"
    if [ -z "$value" ] || [ "$value" = "null" ]; then
        echo '{"none": []}'
    else
        # Escape quotes and format as {"some": "value"}
        local escaped=$(echo "$value" | sed 's/"/\\"/g')
        echo "{\"some\": \"${escaped}\"}"
    fi
}

# =============================================================================
# Seed Categories
# =============================================================================
echo -e "\n${GREEN}[1/5] Seeding Categories...${NC}"

category_count=$(jq '.categories | length' "${SEED_FILE}")
for i in $(seq 0 $((category_count - 1))); do
    id=$(jq -r ".categories[$i].id" "${SEED_FILE}")
    name=$(jq -r ".categories[$i].name" "${SEED_FILE}")
    description=$(jq -r ".categories[$i].description" "${SEED_FILE}")
    display_order=$(jq -r ".categories[$i].display_order" "${SEED_FILE}")
    
    echo -e "  ${YELLOW}→${NC} seed_category: ${name} (id: ${id})"
    spacetime call "${MODULE_NAME}" seed_category -- "${id}" "\"${name}\"" "\"${description}\"" "${display_order}" 2>&1 || {
        echo -e "  ${RED}✗ Failed${NC}"
    }
done
echo -e "${GREEN}✓ ${category_count} categories created${NC}"

# =============================================================================
# Seed Equipment
# =============================================================================
echo -e "\n${GREEN}[2/5] Seeding Equipment...${NC}"

equipment_count=$(jq '.equipment | length' "${SEED_FILE}")
for i in $(seq 0 $((equipment_count - 1))); do
    id=$(jq -r ".equipment[$i].id" "${SEED_FILE}")
    name=$(jq -r ".equipment[$i].name" "${SEED_FILE}")
    description_raw=$(jq -r ".equipment[$i].description // empty" "${SEED_FILE}")
    description_opt=$(format_option "$description_raw")
    
    echo -e "  ${YELLOW}→${NC} seed_equipment: ${name} (id: ${id})"
    spacetime call "${MODULE_NAME}" seed_equipment -- "${id}" "\"${name}\"" "${description_opt}" 2>&1 || {
        echo -e "  ${RED}✗ Failed${NC}"
    }
done
echo -e "${GREEN}✓ ${equipment_count} equipment items created${NC}"

# =============================================================================
# Seed Activities
# =============================================================================
echo -e "\n${GREEN}[3/5] Seeding Activities...${NC}"

activity_count=$(jq '.activities | length' "${SEED_FILE}")
for i in $(seq 0 $((activity_count - 1))); do
    id=$(jq -r ".activities[$i].id" "${SEED_FILE}")
    category_id=$(jq -r ".activities[$i].category_id" "${SEED_FILE}")
    kind=$(jq -r ".activities[$i].kind" "${SEED_FILE}")
    name=$(jq -r ".activities[$i].name" "${SEED_FILE}")
    description=$(jq -r ".activities[$i].description" "${SEED_FILE}")
    instructions=$(jq -r ".activities[$i].instructions" "${SEED_FILE}")
    video_url_raw=$(jq -r ".activities[$i].video_url // empty" "${SEED_FILE}")
    video_url_opt=$(format_option "$video_url_raw")
    xp_required=$(jq -r ".activities[$i].xp_required" "${SEED_FILE}")
    xp_reward=$(jq -r ".activities[$i].xp_reward" "${SEED_FILE}")
    
    # Format the kind enum as SpacetimeDB expects it
    kind_json="{\"${kind}\": {}}"
    
    echo -e "  ${YELLOW}→${NC} seed_activity: ${name} (id: ${id})"
    spacetime call "${MODULE_NAME}" seed_activity -- \
        "${id}" "${category_id}" "${kind_json}" "\"${name}\"" "\"${description}\"" "\"${instructions}\"" "${video_url_opt}" "${xp_required}" "${xp_reward}" 2>&1 || {
        echo -e "  ${RED}✗ Failed${NC}"
    }
done
echo -e "${GREEN}✓ ${activity_count} activities created${NC}"

# =============================================================================
# Seed Activity Prerequisites
# =============================================================================
echo -e "\n${GREEN}[4/5] Seeding Activity Prerequisites...${NC}"

prereq_count=$(jq '.activity_prerequisites | length' "${SEED_FILE}")
for i in $(seq 0 $((prereq_count - 1))); do
    activity_id=$(jq -r ".activity_prerequisites[$i].activity_id" "${SEED_FILE}")
    prerequisite_id=$(jq -r ".activity_prerequisites[$i].prerequisite_id" "${SEED_FILE}")
    
    echo -e "  ${YELLOW}→${NC} seed_prerequisite: Activity ${activity_id} requires ${prerequisite_id}"
    spacetime call "${MODULE_NAME}" seed_prerequisite -- "${activity_id}" "${prerequisite_id}" 2>&1 || {
        echo -e "  ${RED}✗ Failed${NC}"
    }
done
echo -e "${GREEN}✓ ${prereq_count} prerequisites created${NC}"

# =============================================================================
# Seed Activity Equipment
# =============================================================================
echo -e "\n${GREEN}[5/5] Seeding Activity Equipment Links...${NC}"

equip_count=$(jq '.activity_equipment | length' "${SEED_FILE}")
for i in $(seq 0 $((equip_count - 1))); do
    activity_id=$(jq -r ".activity_equipment[$i].activity_id" "${SEED_FILE}")
    equipment_id=$(jq -r ".activity_equipment[$i].equipment_id" "${SEED_FILE}")
    notes_raw=$(jq -r ".activity_equipment[$i].notes // empty" "${SEED_FILE}")
    notes_opt=$(format_option "$notes_raw")
    
    echo -e "  ${YELLOW}→${NC} seed_activity_equipment: Activity ${activity_id} + Equipment ${equipment_id}"
    spacetime call "${MODULE_NAME}" seed_activity_equipment -- "${activity_id}" "${equipment_id}" "${notes_opt}" 2>&1 || {
        echo -e "  ${RED}✗ Failed${NC}"
    }
done
echo -e "${GREEN}✓ ${equip_count} equipment links created${NC}"

# =============================================================================
# Summary
# =============================================================================
echo -e "\n${GREEN}=== Seeding Complete ===${NC}"
echo "Summary:"
echo "  - Categories: ${category_count}"
echo "  - Equipment: ${equipment_count}"
echo "  - Activities: ${activity_count}"
echo "  - Prerequisites: ${prereq_count}"
echo "  - Equipment Links: ${equip_count}"
echo ""
echo -e "${GREEN}All data has been seeded successfully!${NC}"
