/**
 * Seed Activities and Equipment Data into SpacetimeDB
 * 
 * This script can be used in browser or Node.js to seed the database
 * with activities and equipment using the SpacetimeDB client.
 * 
 * Usage in browser console (when connected to SpacetimeDB):
 *   // Copy-paste the seedData object and call seedAll(client)
 * 
 * Usage in Node.js:
 *   node seed_activities.js
 */

const seedData = {
  categories: [
    { name: "Icebreakers", description: "Light introductory activities to warm up and build comfort", display_order: 1 },
    { name: "Trust Building", description: "Activities focused on establishing trust and connection between participants", display_order: 2 },
    { name: "Sensory Exploration", description: "Activities exploring different senses and sensations", display_order: 3 },
    { name: "Challenges", description: "More advanced activities that push comfort zones", display_order: 4 },
    { name: "Creative Expression", description: "Activities focused on artistic and creative exploration", display_order: 5 },
    { name: "Physical Play", description: "Activities involving movement and physical interaction", display_order: 6 }
  ],

  equipment: [
    { name: "Blindfold", description: "Soft eye covering for sensory deprivation activities" },
    { name: "Rope (Soft)", description: "Soft cotton or silk rope, 20-30 feet length" },
    { name: "Timer", description: "Countdown timer or phone with timer app" },
    { name: "Ice Cubes", description: "Regular ice cubes for temperature play" },
    { name: "Feather", description: "Large soft feather for sensation play" },
    { name: "Candles (Massage)", description: "Low-temperature massage candles designed for skin" },
    { name: "Mirror", description: "Full-length or handheld mirror" },
    { name: "Music Player", description: "Device capable of playing music or ambient sounds" },
    { name: "Notebook & Pen", description: "For written activities and journaling" },
    { name: "Dice", description: "Standard six-sided dice for random selection" },
    { name: "Soft Restraints", description: "Padded wrist or ankle cuffs with quick-release" },
    { name: "Massage Oil", description: "Body-safe massage oil or lotion" },
    { name: "Playing Cards", description: "Standard 52-card deck" },
    { name: "Pillows/Cushions", description: "Comfortable pillows for positioning" },
    { name: "Bell", description: "Small bell for non-verbal communication" }
  ],

  activities: [
    { category_id: 1, kind: "Skill", name: "Eye Contact Practice", description: "Hold meaningful eye contact with your partner", instructions: "Sit facing each other in a comfortable position. Set a timer for 2 minutes. Maintain gentle, soft eye contact without speaking. Notice the thoughts and feelings that arise. When the timer ends, share your experience with each other.", video_url: null, xp_required: 0, xp_reward: 10 },
    { category_id: 1, kind: "Skill", name: "Breath Synchronization", description: "Match your breathing rhythm with your partner", instructions: "Sit or lie comfortably near your partner. One person leads by breathing audibly and slowly. The other follows, matching inhales and exhales. After 3 minutes, switch who leads. End with 2 minutes of synchronized breathing without a leader.", video_url: null, xp_required: 0, xp_reward: 15 },
    { category_id: 1, kind: "Skill", name: "Safe Word Establishment", description: "Establish and practice safe words together", instructions: "Discuss and agree on safe words using the traffic light system: GREEN (more/continue), YELLOW (slow down/check in), RED (full stop). Practice saying each word out loud. Role-play scenarios where you might use them. Establish that RED always means immediate pause without question.", video_url: null, xp_required: 0, xp_reward: 20 },
    { category_id: 2, kind: "Skill", name: "Guided Fall", description: "Practice catching and supporting your partner", instructions: "Stand behind your partner. Have them cross their arms over their chest. They will fall backward, trusting you to catch them. Start with small falls, gradually increasing. Always maintain secure footing and communicate clearly before each fall.", video_url: null, xp_required: 25, xp_reward: 25 },
    { category_id: 2, kind: "Activity", name: "Blindfolded Navigation", description: "Guide your blindfolded partner through a space", instructions: "One partner puts on the blindfold. The guide leads them around a familiar room using only verbal instructions and gentle touch guidance. Navigate around obstacles, change directions, and vary speed. The blindfolded person focuses on trusting their partner's guidance.", video_url: null, xp_required: 30, xp_reward: 30 },
    { category_id: 3, kind: "Skill", name: "Light Touch Mapping", description: "Explore sensation with light fingertip touch", instructions: "The receiving partner closes their eyes or wears a blindfold. The giving partner uses only fingertips to trace slowly across their partner's skin. Map different areas: arms, back, neck. The receiver communicates which touches feel most pleasant or sensitive.", video_url: null, xp_required: 20, xp_reward: 20 },
    { category_id: 3, kind: "Activity", name: "Temperature Play Introduction", description: "Explore hot and cold sensations on the skin", instructions: "Gather ice cubes and warm massage oil. The receiver lies comfortably with eyes closed. Alternate between cold (ice cube trails) and warm (heated oil massage). Focus on arms and back. Communicate throughout about sensation levels.", video_url: null, xp_required: 50, xp_reward: 40 },
    { category_id: 3, kind: "Activity", name: "Feather Sensation Journey", description: "Explore ticklish and soft sensations with a feather", instructions: "The receiver lies down and closes their eyes. Using a large soft feather, the giver traces patterns across exposed skin. Vary pressure from barely-there to firm strokes. Cover arms, back, sides, and feet. Take at least 15 minutes to explore fully.", video_url: null, xp_required: 40, xp_reward: 35 },
    { category_id: 4, kind: "Activity", name: "Vulnerability Sharing", description: "Share personal vulnerabilities in a safe space", instructions: "Take turns sharing something you find vulnerable or difficult to talk about. The listener practices active listening without judgment or advice-giving. After sharing, the listener thanks their partner for their trust. Each person shares at least two things.", video_url: null, xp_required: 60, xp_reward: 50 },
    { category_id: 4, kind: "Activity", name: "Time-Limited Stillness", description: "Remain completely still for an extended period", instructions: "The participant takes a comfortable position. Set a timer for 5 minutes (increase as you advance). Remain completely still without speaking, adjusting, or moving. Focus on breath and internal sensations. Partner may watch or participate silently.", video_url: null, xp_required: 75, xp_reward: 45 },
    { category_id: 5, kind: "Skill", name: "Body Appreciation Writing", description: "Write appreciation notes about your partner's body", instructions: "Each partner writes 5 things they appreciate about the other's body. Focus on specific, genuine observations. Read your notes aloud to each other. Take turns, maintaining eye contact while your notes are read to you.", video_url: null, xp_required: 35, xp_reward: 30 },
    { category_id: 5, kind: "Activity", name: "Partner Portrait Session", description: "Take artistic photos or drawings of your partner", instructions: "One partner is the subject, the other the artist. The artist directs poses and captures images or creates sketches. Focus on what you find beautiful about your subject. After 20 minutes, switch roles. Share and discuss your creations.", video_url: null, xp_required: 45, xp_reward: 40 },
    { category_id: 6, kind: "Skill", name: "Partner Stretching", description: "Assist your partner in gentle stretches", instructions: "One partner guides the other through assisted stretches. Apply gentle, gradual pressure. Focus on legs, back, and shoulders. Communicate constantly about comfort levels. Hold each stretch for 30 seconds. Switch roles after 10 minutes.", video_url: null, xp_required: 15, xp_reward: 20 },
    { category_id: 6, kind: "Activity", name: "Dance Without Music", description: "Move together in synchronized movement without music", instructions: "Stand facing each other. One person leads with slow movements, the other mirrors exactly. Move gradually, allowing your partner to follow. After 5 minutes, switch leaders. Finally, try 5 minutes without a designated leader.", video_url: null, xp_required: 55, xp_reward: 35 },
    { category_id: 2, kind: "Activity", name: "Wrist Binding Introduction", description: "Introduction to consensual wrist restraint with soft restraints", instructions: "Discuss boundaries and confirm consent. Apply soft restraints loosely to wrists in front of the body. Check for circulation and comfort. Stay in position for 5 minutes while breathing calmly. Remove immediately upon request. Discuss the experience afterward.", video_url: null, xp_required: 100, xp_reward: 60 },
    { category_id: 3, kind: "Activity", name: "Massage Journey", description: "Give a complete relaxation massage to your partner", instructions: "Prepare a warm, comfortable space with massage oil. The receiver lies face down. Begin with back, using long flowing strokes. Move to arms, legs, and feet. Maintain consistent pressure and rhythm. Take at least 30 minutes. The receiver focuses only on receiving and relaxing.", video_url: null, xp_required: 70, xp_reward: 55 },
    { category_id: 4, kind: "Activity", name: "Card Game of Dares", description: "Draw cards to determine escalating challenges", instructions: "Assign meaning to card values: 2-4 (mild), 5-7 (medium), 8-10 (intense), face cards (partner's choice). Take turns drawing cards and performing the corresponding challenge. Agree on challenge definitions beforehand. Always honor safe words.", video_url: null, xp_required: 85, xp_reward: 50 },
    { category_id: 1, kind: "Skill", name: "Verbal Check-In Practice", description: "Practice clear communication during activities", instructions: "During any activity, practice regular check-ins. Use phrases like 'How are you feeling?', 'Rate this 1-10?', 'More or less?'. The receiver practices giving clear, honest answers. Make checking in feel natural and supportive, not interrupting.", video_url: null, xp_required: 10, xp_reward: 15 },
    { category_id: 5, kind: "Activity", name: "Sensory Journaling", description: "Write about physical sensations experienced together", instructions: "After any shared activity, each person spends 10 minutes writing about the sensations experienced. Describe textures, temperatures, emotions, and physical responses. Share your writing with each other. Discuss similarities and differences in experience.", video_url: null, xp_required: 40, xp_reward: 30 },
    { category_id: 6, kind: "Activity", name: "Slow Dance Connection", description: "Dance slowly and intimately with your partner", instructions: "Put on slow, sensual music. Stand close with full-body contact. Move slowly together, focusing on the connection rather than steps. Maintain eye contact when possible. Dance for at least 2-3 songs without speaking.", video_url: null, xp_required: 30, xp_reward: 25 }
  ],

  // Prerequisites reference activity indexes (1-based as they appear in activities array)
  activity_prerequisites: [
    { activity_id: 5, prerequisite_id: 3 },   // Blindfolded Navigation requires Safe Word Establishment
    { activity_id: 7, prerequisite_id: 6 },   // Temperature Play requires Light Touch Mapping
    { activity_id: 7, prerequisite_id: 3 },   // Temperature Play requires Safe Word Establishment
    { activity_id: 8, prerequisite_id: 6 },   // Feather Sensation requires Light Touch Mapping
    { activity_id: 9, prerequisite_id: 1 },   // Vulnerability Sharing requires Eye Contact Practice
    { activity_id: 9, prerequisite_id: 18 },  // Vulnerability Sharing requires Verbal Check-In Practice
    { activity_id: 10, prerequisite_id: 2 },  // Time-Limited Stillness requires Breath Synchronization
    { activity_id: 14, prerequisite_id: 1 },  // Dance Without Music requires Eye Contact Practice
    { activity_id: 15, prerequisite_id: 3 },  // Wrist Binding requires Safe Word Establishment
    { activity_id: 15, prerequisite_id: 4 },  // Wrist Binding requires Guided Fall
    { activity_id: 16, prerequisite_id: 6 },  // Massage Journey requires Light Touch Mapping
    { activity_id: 16, prerequisite_id: 13 }, // Massage Journey requires Partner Stretching
    { activity_id: 17, prerequisite_id: 3 },  // Card Game of Dares requires Safe Word Establishment
    { activity_id: 17, prerequisite_id: 9 },  // Card Game of Dares requires Vulnerability Sharing
    { activity_id: 20, prerequisite_id: 1 },  // Slow Dance Connection requires Eye Contact Practice
    { activity_id: 20, prerequisite_id: 2 }   // Slow Dance Connection requires Breath Synchronization
  ],

  // Equipment links reference activity and equipment indexes (1-based)
  activity_equipment: [
    { activity_id: 2, equipment_id: 3, notes: "For timing breathing intervals" },
    { activity_id: 5, equipment_id: 1, notes: "Essential for the activity" },
    { activity_id: 6, equipment_id: 1, notes: "Optional - can also just close eyes" },
    { activity_id: 7, equipment_id: 4, notes: "For cold sensation" },
    { activity_id: 7, equipment_id: 12, notes: "Warm the oil for heat contrast" },
    { activity_id: 7, equipment_id: 1, notes: "Heightens sensation awareness" },
    { activity_id: 8, equipment_id: 5, notes: "Main tool for this activity" },
    { activity_id: 8, equipment_id: 1, notes: "Optional - enhances sensation focus" },
    { activity_id: 10, equipment_id: 3, notes: "For timing the stillness period" },
    { activity_id: 10, equipment_id: 14, notes: "For comfortable positioning" },
    { activity_id: 11, equipment_id: 9, notes: "For writing appreciation notes" },
    { activity_id: 12, equipment_id: 7, notes: "Optional - for the subject to see themselves" },
    { activity_id: 15, equipment_id: 11, notes: "Essential - use only quick-release restraints" },
    { activity_id: 15, equipment_id: 3, notes: "For timing the experience" },
    { activity_id: 15, equipment_id: 15, notes: "For non-verbal signaling if needed" },
    { activity_id: 16, equipment_id: 12, notes: "Essential for the massage" },
    { activity_id: 16, equipment_id: 14, notes: "For proper positioning and comfort" },
    { activity_id: 16, equipment_id: 8, notes: "Optional - ambient music enhances relaxation" },
    { activity_id: 17, equipment_id: 13, notes: "Essential for card drawing" },
    { activity_id: 17, equipment_id: 10, notes: "Optional - can add randomness to challenges" },
    { activity_id: 19, equipment_id: 9, notes: "For writing journal entries" },
    { activity_id: 20, equipment_id: 8, notes: "For playing slow music" }
  ]
};

/**
 * Seed all data using SpacetimeDB client reducers
 * @param {Object} client - SpacetimeDB client with reducer access
 */
async function seedAll(client) {
  console.log("=== SpacetimeDB Activity & Equipment Seeder ===\n");
  
  // Track created IDs for relationship mapping
  const categoryIds = new Map();
  const equipmentIds = new Map();
  const activityIds = new Map();
  
  // 1. Seed Categories
  console.log("[1/5] Seeding Categories...");
  for (let i = 0; i < seedData.categories.length; i++) {
    const cat = seedData.categories[i];
    try {
      await client.reducers.admin_create_category(
        cat.name,
        cat.description,
        cat.display_order
      );
      categoryIds.set(i + 1, true);
      console.log(`  ✓ ${cat.name}`);
    } catch (err) {
      console.error(`  ✗ ${cat.name}: ${err.message}`);
    }
  }
  
  // 2. Seed Equipment
  console.log("\n[2/5] Seeding Equipment...");
  for (let i = 0; i < seedData.equipment.length; i++) {
    const eq = seedData.equipment[i];
    try {
      await client.reducers.admin_create_equipment(
        eq.name,
        eq.description || null
      );
      equipmentIds.set(i + 1, true);
      console.log(`  ✓ ${eq.name}`);
    } catch (err) {
      console.error(`  ✗ ${eq.name}: ${err.message}`);
    }
  }
  
  // 3. Seed Activities
  console.log("\n[3/5] Seeding Activities...");
  for (let i = 0; i < seedData.activities.length; i++) {
    const act = seedData.activities[i];
    try {
      await client.reducers.admin_create_activity(
        act.category_id,
        act.kind,
        act.name,
        act.description,
        act.instructions,
        act.video_url,
        act.xp_required,
        act.xp_reward
      );
      activityIds.set(i + 1, true);
      console.log(`  ✓ ${act.name} (${act.kind})`);
    } catch (err) {
      console.error(`  ✗ ${act.name}: ${err.message}`);
    }
  }
  
  // 4. Seed Prerequisites
  console.log("\n[4/5] Seeding Prerequisites...");
  for (const prereq of seedData.activity_prerequisites) {
    try {
      await client.reducers.admin_add_prerequisite(
        prereq.activity_id,
        prereq.prerequisite_id
      );
      console.log(`  ✓ Activity ${prereq.activity_id} requires ${prereq.prerequisite_id}`);
    } catch (err) {
      console.error(`  ✗ ${prereq.activity_id} → ${prereq.prerequisite_id}: ${err.message}`);
    }
  }
  
  // 5. Seed Activity Equipment
  console.log("\n[5/5] Seeding Activity Equipment...");
  for (const ae of seedData.activity_equipment) {
    try {
      await client.reducers.admin_add_activity_equipment(
        ae.activity_id,
        ae.equipment_id,
        ae.notes || null
      );
      console.log(`  ✓ Activity ${ae.activity_id} uses Equipment ${ae.equipment_id}`);
    } catch (err) {
      console.error(`  ✗ Activity ${ae.activity_id} + Equipment ${ae.equipment_id}: ${err.message}`);
    }
  }
  
  console.log("\n=== Seeding Complete ===");
  console.log(`Summary:
  - Categories: ${seedData.categories.length}
  - Equipment: ${seedData.equipment.length}
  - Activities: ${seedData.activities.length}
  - Prerequisites: ${seedData.activity_prerequisites.length}
  - Equipment Links: ${seedData.activity_equipment.length}
  `);
}

/**
 * Clear all activity-related data
 * Warning: This is destructive!
 * @param {Object} client - SpacetimeDB client with reducer access
 */
async function clearAllActivityData(client) {
  console.log("=== Clearing All Activity Data ===");
  console.log("WARNING: This will delete all activities, equipment, and related data!");
  
  // Get current data from tables
  const activities = Array.from(client.db.activity.iter());
  const equipment = Array.from(client.db.equipment.iter());
  const categories = Array.from(client.db.category.iter());
  
  // Delete activities first (this cascades to prerequisites and equipment links)
  console.log(`\nDeleting ${activities.length} activities...`);
  for (const act of activities) {
    try {
      await client.reducers.admin_delete_activity(act.id);
      console.log(`  ✓ Deleted activity: ${act.name}`);
    } catch (err) {
      console.error(`  ✗ ${act.name}: ${err.message}`);
    }
  }
  
  // Delete equipment
  console.log(`\nDeleting ${equipment.length} equipment items...`);
  for (const eq of equipment) {
    try {
      await client.reducers.admin_delete_equipment(eq.id);
      console.log(`  ✓ Deleted equipment: ${eq.name}`);
    } catch (err) {
      console.error(`  ✗ ${eq.name}: ${err.message}`);
    }
  }
  
  // Delete categories
  console.log(`\nDeleting ${categories.length} categories...`);
  for (const cat of categories) {
    try {
      await client.reducers.admin_delete_category(cat.id);
      console.log(`  ✓ Deleted category: ${cat.name}`);
    } catch (err) {
      console.error(`  ✗ ${cat.name}: ${err.message}`);
    }
  }
  
  console.log("\n=== Clear Complete ===");
}

// Export for use as module
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { seedData, seedAll, clearAllActivityData };
}

// Browser usage hint
if (typeof window !== 'undefined') {
  window.seedActivities = { seedData, seedAll, clearAllActivityData };
  console.log("Seed functions available:");
  console.log("  - window.seedActivities.seedAll(client)");
  console.log("  - window.seedActivities.clearAllActivityData(client)");
  console.log("  - window.seedActivities.seedData (raw data)");
}

