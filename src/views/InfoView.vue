<script setup>
import {
  mdiLightningBolt,
  mdiSwapHorizontal,
  mdiShieldLock,
  mdiCogs,
  mdiChartBar,
  mdiScaleBalance,
  mdiSpeedometer,
  mdiBullseyeArrow,
  mdiShieldCheck,
  mdiGestureTap,
  mdiInformationOutline,
  mdiHandPointingUp,
  mdiLibraryOutline,
  mdiSeal,
} from "@mdi/js";

const glossary = [
  { term: "Fast Mana", icon: mdiLightningBolt, color: "amber", desc: "Permanents that produce more mana than they cost. High-velocity fuel." },
  { term: "Rituals & Bursts", icon: mdiLightningBolt, color: "orange", desc: "One-shot mana like rituals and Treasure. Essential for Turbo strategies." },
  { term: "Tutors", icon: mdiBullseyeArrow, color: "blue", desc: "Cards that search for other cards. Increases deck Consistency." },
  { term: "Engines", icon: mdiCogs, color: "purple", desc: "Repeatable value sources. Key for Midrange and Commander-focused decks." },
  { term: "Interaction", icon: mdiGestureTap, color: "red", desc: "Removal and counterspells. Essential for controlling the game state." },
  { term: "Stax", icon: mdiShieldLock, color: "deep-orange", desc: "Resource denial. Slows down the entire table." },
  { term: "Protection", icon: mdiShieldCheck, color: "green", desc: "Hexproof, Indestructible, or recursion. Increases deck Resilience." },
  { term: "Wincons", icon: mdiSeal, color: "success", desc: "Cards that directly lead to winning. Compact wincons boost Speed." },
];

const archetypes = [
  {
    title: "Turbo",
    icon: mdiLightningBolt,
    color: "amber",
    definition: "High-velocity strategy focusing on winning as fast as possible.",
    traits: [
      "Extensive use of low-cost fast mana and rituals",
      "High density of burst draw and mass draw effects",
      "Signal Floor: ≥ 12.0 Speed (Explosive Mana + Draw)",
      "Designed to attempt a win in the early game (Turns 1-4)",
    ],
    strategy: "Win before opponents can establish interaction or board state.",
  },
  {
    title: "Midrange",
    icon: mdiSwapHorizontal,
    color: "blue",
    definition: "Flexible strategy that balances value engines with interaction.",
    traits: [
      "Consistent Tutors and steady Draw Engines",
      "High Interaction (at least 6 pieces ≤ 2 MV)",
      "High Pivotability (versatile cards)",
      "Out-values and out-interacts opponents over time",
    ],
    strategy: "Adapt to the game; switch between defense and offense as needed.",
  },
  {
    title: "Stax",
    icon: mdiShieldLock,
    color: "deep-orange",
    definition: "Control strategy focusing on resource denial and game-slowing pieces.",
    traits: [
      "Presence of resource denial and taxing effects",
      "High Resilience to protect permanents",
      "Signal Floor: ≥ 10.0 Stax signal",
      "Wins by establishing a lock and exhausting opponents",
    ],
    strategy: "Force the game to your speed; win while others are resource-starved.",
  },
  {
    title: "Commander Engine",
    icon: mdiCogs,
    color: "purple",
    definition: "Strategy built entirely around the value provided by the Commander.",
    traits: [
      "Commander serves as a primary source of card advantage or efficiency",
      "Relies on Command Zone access for core strategy",
      "High Consistency through repeatable effects",
      "Signal Floor: Commander provides card-based engine signal",
    ],
    strategy: "Protect the Commander and leverage the Command Zone for infinite value.",
  },
];

const dimensions = [
  { name: "Consistency (C)", icon: mdiBullseyeArrow, desc: "Weights Tutors, Draw, and Engines. Thresholds: 1.2, 3.0, 5.0, 8.0.", color: "blue" },
  { name: "Resilience (R)", icon: mdiShieldCheck, desc: "Weights Protection and Recursion. Thresholds: 1.5, 4.0, 7.0, 10.0.", color: "green" },
  { name: "Interaction (I)", icon: mdiGestureTap, desc: "Weights Removal and Stax. Thresholds: 2.5, 5.0, 8.0, 12.0.", color: "red" },
  { name: "Speed (S)", icon: mdiSpeedometer, desc: "Calculated from estimated win turn (Efficiency) and high-velocity signals (Explosive Mana and Draw).", color: "amber" },
  { name: "Pivotability (P)", icon: mdiScaleBalance, desc: "Weights multi-role cards. Thresholds: 1.5, 3.0, 5.0, 8.0.", color: "purple" },
];

const structuralMetrics = [
  { name: "Role Score", desc: "A normalized measure of the deck's raw role coverage (Raw Score / 25.0)." },
  { name: "Land Score", desc: "Measures the adequacy of the land count, normalized against a standard 38-land baseline." },
  { name: "Commander MV Penalty", desc: "A penalty applied based on the total mana value of your commanders, reflecting the cost of accessing the command zone." },
];

const brackets = [
  { score: "≥ 23.0", tier: "cEDH Optimized", desc: "The highest tier of competitive play. These decks are built with zero compromises, utilizing the most efficient mana, tutors, and win conditions available in the format." },
  { score: "≥ 19.0", tier: "Fringe cEDH", desc: "Highly competitive decks that may be slightly slower or less consistent than top-tier options, often due to choosing a less optimal commander or a more niche strategy." },
  { score: "≥ 15.0", tier: "High Power", desc: "Decks with powerful synergies and reliable win conditions. They lack the extreme speed or density of free interaction found in cEDH but can easily dominate casual tables." },
  { score: "≥ 9.0", tier: "Focused / Synergistic", desc: "Decks with a clear game plan and decent support. They may lack expensive staples or highly efficient combos but have a cohesive strategy." },
  { score: "< 9.0", tier: "Casual / Battlecruiser", desc: "Lower-powered decks that focus on longer games, combat damage, or theme-heavy builds. These decks typically lack efficient tutors and fast mana." },
];

const numericalBrackets = [
  { level: "Bracket 5", condition: "Score ≥ 24.0", desc: "Apex competitive decks." },
  { level: "Bracket 4", condition: "GC > 3 OR Infinite Combo Detected", desc: "Decks with high-impact game changers or deterministic wincons." },
  { level: "Bracket 3", condition: "GC ≤ 3", desc: "Decks with moderate game-changing potential." },
  { level: "Bracket 2", condition: "Default / Baseline", desc: "Standard deck configuration." },
  { level: "Bracket 1", condition: "Score ≤ 8.0 AND AMV > 3.5", desc: "Decks significantly constrained by high mana costs and low efficiency." },
];

const qualityTiers = [
  { name: "Premium", weight: "1.5", desc: "Maximum efficiency pieces: 0-1 mana interaction, low-cost tutors, and high-velocity fast mana." },
  { name: "Efficient", weight: "1.0", desc: "Standard high-quality cards: 2-mana interaction, reliable engines, and solid utility lands." },
  { name: "Slow", weight: "0.75", desc: "Higher-cost alternatives or cards that require significant setup to be effective." },
  { name: "Low Impact", weight: "0.5", desc: "Cards with marginal utility or those that do not contribute significantly to any core CRISPI dimension." },
];

const floorLogic = [
  { 
    name: "Structural Overrides", 
    desc: "Safety nets that ensure decks with high-quality components meet a minimum performance floor, regardless of raw quantity.",
    items: [
      { condition: "8+ Free Interaction Spells", effect: "Interaction = 5, Resilience ≥ 4", color: "red" },
      { condition: "5+ Premium Tutors (≤ 2 MV)", effect: "Consistency = 5", color: "blue" },
      { condition: "10+ Fast Mana Pieces", effect: "Speed = 5", color: "amber" },
      { condition: "Compact Win Package (Efficient wincons ≥ 2)", effect: "Speed = 5, Consistency +1", color: "success" },
      { condition: "Commander is a Card Advantage Engine", effect: "Consistency ≥ 4, Pivotability ≥ 4", color: "purple" },
    ]
  },
  {
    name: "Archetype Floors",
    desc: "Minimum dimension scores required to maintain the identity of a specific strategy.",
    items: [
      { condition: "Turbo Archetype", effect: "Consistency ≥ 4, Pivotability ≥ 3", color: "amber" },
      { condition: "Stax Archetype", effect: "Interaction ≥ 4, Resilience ≥ 4", color: "deep-orange" },
      { condition: "Commander Engine Archetype", effect: "Consistency ≥ 4, Resilience ≥ 3, Pivotability ≥ 3", color: "purple" },
    ]
  }
];

const scoringBonuses = [
  { name: "Game Changer Count (GC)", impact: "+0.4 per GC", desc: "Direct addition to raw score (max +6.0). Reflects high-impact cards that can single-handedly shift game momentum." },
  { name: "CMDR MV Penalty", impact: "-0.05 per MV", desc: "Deduction based on total Commander mana value. Reflects the resource tax of accessing your command zone strategy." },
  { name: "Land Balance", impact: "0.0 to 1.0", desc: "A structural health score based on land count relative to a 38-land baseline. This is factored into the role score." },
];
</script>

<template>
  <v-container class="info-view pb-12">
    <v-row>
      <v-col cols="12">
        <h1 class="text-h3 font-weight-bold mb-2 text-primary">CRISPI Evaluation</h1>
        <p class="text-subtitle-1 text-medium-emphasis mb-6">
          A comprehensive framework for analyzing Magic: The Gathering decks based on strategy, speed, and efficiency.
        </p>

        <!-- Evaluation Process -->
        <v-alert color="primary" variant="tonal" border="start" class="mb-8" :icon="mdiInformationOutline">
          <h3 class="text-h6 mb-2">Evaluation Process</h3>
          <p class="text-body-2 mb-4">
            CRISPI performs a multi-stage analysis of your deck to determine its power level and strategy.
          </p>
          <v-row>
            <v-col cols="12" sm="4">
              <div class="d-flex align-center mb-2">
                <v-avatar color="primary" size="x-small" class="mr-2">1</v-avatar>
                <span class="font-weight-bold">Role Identification</span>
              </div>
              <p class="text-caption text-medium-emphasis">Every card is assigned roles (Mana, Draw, Interaction, etc.) based on its oracle text and properties.</p>
            </v-col>
            <v-col cols="12" sm="4">
              <div class="d-flex align-center mb-2">
                <v-avatar color="primary" size="x-small" class="mr-2">2</v-avatar>
                <span class="font-weight-bold">Weighted Scoring</span>
              </div>
              <p class="text-caption text-medium-emphasis">Roles are weighted by card quality (Premium to Low Impact) and used to score the five CRISPI dimensions.</p>
            </v-col>
            <v-col cols="12" sm="4">
              <div class="d-flex align-center mb-2">
                <v-avatar color="primary" size="x-small" class="mr-2">3</v-avatar>
                <span class="font-weight-bold">Global Multipliers</span>
              </div>
              <p class="text-caption text-medium-emphasis">Average Mana Value (AMV) and detected Infinite Combos apply multipliers to the final score.</p>
            </v-col>
          </v-row>
        </v-alert>
      </v-col>
    </v-row>

    <!-- Glossary of Roles -->
    <v-row class="mb-8">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiLibraryOutline" class="mr-2" color="primary"></v-icon>
          Glossary of Roles
        </h2>
        <v-divider class="mb-6"></v-divider>
        <p class="text-body-1 mb-6">
          CRISPI uses specific terms to categorize your cards. Here is what they mean in plain English:
        </p>
      </v-col>
      <v-col v-for="item in glossary" :key="item.term" cols="12" sm="6" md="4" lg="2.4">
        <v-card variant="tonal" :color="item.color" class="h-100 pa-4 text-center">
          <v-icon :icon="item.icon" size="large" class="mb-2"></v-icon>
          <div class="text-subtitle-1 font-weight-bold mb-1">{{ item.term }}</div>
          <div class="text-caption">{{ item.desc }}</div>
        </v-card>
      </v-col>
    </v-row>

    <!-- Deck Archetypes -->
    <v-row class="mb-8">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiChartBar" class="mr-2" color="primary"></v-icon>
          Deck Archetypes
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>

      <v-col v-for="arc in archetypes" :key="arc.title" cols="12" md="6">
        <v-card variant="outlined" class="h-100 archetype-card">
          <v-card-item>
            <template #prepend>
              <v-avatar :color="arc.color" variant="tonal" class="mr-3">
                <v-icon :icon="arc.icon"></v-icon>
              </v-avatar>
            </template>
            <v-card-title class="text-h5">{{ arc.title }}</v-card-title>
          </v-card-item>

          <v-card-text>
            <p class="mb-4">{{ arc.definition }}</p>
            
            <div class="mb-4">
              <div class="text-subtitle-2 font-weight-bold mb-1">Key Traits:</div>
              <ul class="pl-5">
                <li v-for="trait in arc.traits" :key="trait" class="mb-1">{{ trait }}</li>
              </ul>
            </div>

            <v-alert
              density="compact"
              variant="tonal"
              :color="arc.color"
              :icon="false"
              class="mt-auto"
            >
              <strong>Strategy:</strong> {{ arc.strategy }}
            </v-alert>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- CRISPI Dimensions -->
    <v-row class="mb-8">
      <v-col cols="12">
        <h2 class="text-h4 mb-4">The CRISPI Dimensions</h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col v-for="dim in dimensions" :key="dim.name" cols="12" sm="6" md="4" lg="2.4">
        <v-card variant="flat" border class="h-100 text-center py-4 px-2" :style="{ borderColor: `rgb(var(--v-theme-${dim.color})) !important` }">
          <v-icon :icon="dim.icon" size="x-large" :color="dim.color" class="mb-2"></v-icon>
          <v-card-title class="text-subtitle-1 font-weight-bold pb-0">{{ dim.name }}</v-card-title>
          <v-card-text class="text-caption">{{ dim.desc }}</v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Structural Metrics -->
    <v-row class="mb-8">
      <v-col cols="12">
        <h2 class="text-h4 mb-4">Structural Metrics</h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col v-for="metric in structuralMetrics" :key="metric.name" cols="12" md="4">
        <v-card variant="flat" border class="h-100 pa-4">
          <h3 class="text-subtitle-1 font-weight-bold mb-2 text-primary">{{ metric.name }}</h3>
          <p class="text-body-2 text-medium-emphasis">{{ metric.desc }}</p>
        </v-card>
      </v-col>
    </v-row>

    <!-- Structural Floors & Overrides -->
    <v-row class="mb-8 mt-4">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiShieldCheck" class="mr-2" color="primary"></v-icon>
          CRISPI Floor Logic
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col v-for="section in floorLogic" :key="section.name" cols="12" md="6">
        <v-card variant="outlined" class="pa-4 h-100">
          <h3 class="text-h6 mb-2">{{ section.name }}</h3>
          <p class="text-body-2 text-medium-emphasis mb-4">{{ section.desc }}</p>
          <v-table density="compact">
            <thead>
              <tr>
                <th class="text-left">Condition</th>
                <th class="text-left">Floor / Override</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in section.items" :key="item.condition">
                <td class="font-weight-medium" :style="{ color: `rgb(var(--v-theme-${item.color}))` }">{{ item.condition }}</td>
                <td><v-chip size="x-small" :color="item.color" variant="tonal">{{ item.effect }}</v-chip></td>
              </tr>
            </tbody>
          </v-table>
        </v-card>
      </v-col>
    </v-row>

    <!-- Scoring Brackets & Interpretations -->
    <v-row class="mb-8 mt-4">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiChartBar" class="mr-2" color="primary"></v-icon>
          Power Brackets & Interpretations
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      
      <!-- Power Ranges -->
      <v-col cols="12" md="7">
        <v-card variant="outlined" class="pa-4 h-100">
          <h3 class="text-h6 mb-4">CRISPI Score Interpretations</h3>
          <v-list lines="two" density="compact">
            <v-list-item v-for="range in brackets" :key="range.tier" class="px-0">
              <template #prepend>
                <v-chip class="mr-4" color="primary" label>{{ range.score }}</v-chip>
              </template>
              <v-list-item-title class="font-weight-bold">{{ range.tier }}</v-list-item-title>
              <v-list-item-subtitle class="text-wrap">{{ range.desc }}</v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </v-card>
      </v-col>

      <!-- Numerical Brackets -->
      <v-col cols="12" md="5">
        <v-card variant="outlined" class="pa-4 h-100">
          <h3 class="text-h6 mb-4">Numerical Brackets (1-5)</h3>
          <v-list lines="two" density="compact">
            <v-list-item v-for="num in numericalBrackets" :key="num.level" class="px-0">
              <template #prepend>
                <div class="text-h6 font-weight-black mr-4 text-primary" style="min-width: 80px">{{ num.level }}</div>
              </template>
              <v-list-item-title class="text-caption font-weight-bold">{{ num.condition }}</v-list-item-title>
              <v-list-item-subtitle class="text-wrap">{{ num.desc }}</v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </v-card>
      </v-col>
    </v-row>

    <!-- Two-Card Infinite Combos -->
    <v-row class="mb-8 mt-4">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiGestureTap" class="mr-2" color="primary"></v-icon>
          Infinite Combo Detection
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col cols="12">
        <v-card variant="outlined" class="pa-4">
          <p class="mb-4 text-body-1">
            CRISPI automatically detects two-card infinite combos. Finding a valid combo significantly increases your deck's power score and ensures it is placed in <strong>Bracket 4</strong> or higher.
          </p>
          <v-row>
            <v-col cols="12" md="6">
              <h3 class="text-h6 mb-2">Multiplier Logic</h3>
              <v-list density="compact">
                <v-list-item>
                  <v-list-item-title>Base Bonus (per combo): <span class="text-primary font-weight-bold">+0.15x</span></v-list-item-title>
                  <v-list-item-subtitle>Standard bonus for efficient combos (MV ≤ 3).</v-list-item-subtitle>
                </v-list-item>
                <v-list-item>
                  <v-list-item-title>Tutor Synergy: <span class="text-primary font-weight-bold">+0.05x</span></v-list-item-title>
                  <v-list-item-subtitle>Applied if the deck contains at least one tutor.</v-list-item-subtitle>
                </v-list-item>
                <v-list-item>
                  <v-list-item-title>Total Multiplier Cap: <span class="text-primary font-weight-bold">1.30x</span></v-list-item-title>
                  <v-list-item-subtitle>The maximum total bonus from all detected combos.</v-list-item-subtitle>
                </v-list-item>
              </v-list>
            </v-col>
            <v-col cols="12" md="6">
              <h3 class="text-h6 mb-2">Penalties (Reduces Bonus)</h3>
              <v-list density="compact">
                <v-list-item>
                  <v-list-item-title>Mana Value Penalty: <span class="text-error font-weight-bold">-0.01x per MV</span></v-list-item-title>
                  <v-list-item-subtitle>Applied for each point of combined Mana Value over 3.</v-list-item-subtitle>
                </v-list-item>
                <v-list-item>
                  <v-list-item-title>Speed Penalty (Sorcery): <span class="text-error font-weight-bold">-0.02x</span></v-list-item-title>
                  <v-list-item-subtitle>Applied if a combo piece is a Sorcery or requires sorcery speed.</v-list-item-subtitle>
                </v-list-item>
                <v-list-item>
                  <v-list-item-title>Speed Penalty (Tap): <span class="text-error font-weight-bold">-0.02x</span></v-list-item-title>
                  <v-list-item-subtitle>Applied if a creature/artifact piece needs to tap and lacks Haste.</v-list-item-subtitle>
                </v-list-item>
              </v-list>
            </v-col>
          </v-row>
          <v-alert type="info" variant="tonal" border="start" density="comfortable" class="mt-4">
            A single combo bonus is clamped between <strong>0.05x</strong> and <strong>0.20x</strong> after penalties.
          </v-alert>
        </v-card>
      </v-col>
    </v-row>

    <!-- Why was my deck scored this way? -->
    <v-row class="mb-8 mt-4">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiScaleBalance" class="mr-2" color="primary"></v-icon>
          Scoring FAQ: "Why was my deck scored this way?"
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col cols="12">
        <v-expansion-panels variant="accordion">
          <v-expansion-panel
            title="Why am I registered as 'Midrange' despite having fast mana?"
            text="CRISPI requires a specific 'Signal Floor' to qualify for specialized archetypes. If your deck has a few pieces of fast mana but lacks the critical mass of burst draw or rituals to sustain that velocity, it falls back to Midrange—the most flexible and common category."
          ></v-expansion-panel>
          <v-expansion-panel
            title="How does the Average Mana Value (AMV) affect my score?"
            text="The AMV multiplier is a major efficiency factor. A deck with a low AMV (under 2.0) receives a significant bonus, while a deck with a high AMV (over 3.0) receives a penalty. This reflects the inherent power of being able to cast more spells per turn."
          ></v-expansion-panel>
          <v-expansion-panel
            title="What are 'Structural Floors'?"
            text="Structural floors are safety nets. If you run a high volume of free interaction, CRISPI assumes your deck has a certain level of defensive power and will floor your Interaction score to its maximum, even if the raw quantity is lower. This ensures high-quality interaction is never undervalued."
          ></v-expansion-panel>
          <v-expansion-panel
            title="Why did my score jump after adding a combo?"
            text="Deterministic two-card infinite combos are significant power multipliers. CRISPI detects these automatically. Having a compact win condition that requires minimal setup is the most direct way to move into higher power brackets."
          ></v-expansion-panel>
        </v-expansion-panels>
      </v-col>
    </v-row>

    <!-- Weighting & Quality Tiers -->
    <v-row class="mb-8">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiScaleBalance" class="mr-2" color="primary"></v-icon>
          Card Weighting & Quality Tiers
        </h2>
        <v-divider class="mb-6"></v-divider>
        <p class="text-body-1 mb-6">
          Not all cards are created equal. CRISPI weights each card's contribution to a dimension based on its efficiency and impact.
        </p>
      </v-col>
      <v-col v-for="tier in qualityTiers" :key="tier.name" cols="12" sm="6" md="3">
        <v-card variant="outlined" class="pa-4 h-100">
          <div class="d-flex justify-space-between align-center mb-2">
            <span class="text-subtitle-1 font-weight-bold">{{ tier.name }}</span>
            <v-chip size="x-small" color="primary" variant="flat">{{ tier.weight }}x Weight</v-chip>
          </div>
          <p class="text-caption text-medium-emphasis">{{ tier.desc }}</p>
        </v-card>
      </v-col>
    </v-row>

    <!-- Multipliers & Bonuses -->
    <v-row class="mb-8 mt-4">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiChartBar" class="mr-2" color="primary"></v-icon>
          Multipliers & Bonuses
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col cols="12" md="8">
        <v-card variant="outlined" class="pa-4 h-100">
          <h3 class="text-h6 mb-4">AMV Multiplier (Efficiency)</h3>
          <p class="text-body-2 mb-4">
            The Average Mana Value (AMV) of your non-land cards significantly impacts your final score.
          </p>
          <v-table density="compact">
            <thead>
              <tr>
                <th class="text-left">AMV Range</th>
                <th class="text-left">Multiplier</th>
                <th class="text-left">Impact</th>
              </tr>
            </thead>
            <tbody>
              <tr><td>≤ 1.3</td><td class="font-weight-bold">1.20x</td><td class="text-success">+20% Bonus</td></tr>
              <tr><td>≤ 1.6</td><td class="font-weight-bold">1.12x</td><td class="text-success">+12% Bonus</td></tr>
              <tr><td>≤ 2.0</td><td class="font-weight-bold">1.05x</td><td class="text-success">+5% Bonus</td></tr>
              <tr><td>≤ 2.4</td><td class="font-weight-bold">1.02x</td><td class="text-success">+2% Bonus</td></tr>
              <tr><td>2.5 – 2.8</td><td class="font-weight-bold">1.00x</td><td class="text-medium-emphasis">Neutral</td></tr>
              <tr><td>≤ 3.0</td><td class="font-weight-bold">0.94x</td><td class="text-error">-6% Penalty</td></tr>
              <tr><td>≤ 3.4</td><td class="font-weight-bold">0.88x</td><td class="text-error">-12% Penalty</td></tr>
              <tr><td>≤ 3.8</td><td class="font-weight-bold">0.72x</td><td class="text-error">-28% Penalty</td></tr>
              <tr><td>> 3.8</td><td class="font-weight-bold">0.55x</td><td class="text-error">-45% Penalty</td></tr>
            </tbody>
          </v-table>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card variant="outlined" class="pa-4 h-100">
          <h3 class="text-h6 mb-4">Other Scoring Factors</h3>
          <div v-for="bonus in scoringBonuses" :key="bonus.name" class="mb-4">
            <div class="d-flex justify-space-between align-center mb-1">
              <span class="font-weight-bold">{{ bonus.name }}</span>
              <v-chip size="x-small" color="secondary" variant="tonal">{{ bonus.impact }}</v-chip>
            </div>
            <p class="text-caption text-medium-emphasis">{{ bonus.desc }}</p>
          </div>
        </v-card>
      </v-col>
    </v-row>

    <!-- Pro Tips -->
    <v-row class="mb-8 mt-4">
      <v-col cols="12">
        <h2 class="text-h4 mb-4 d-flex align-center">
          <v-icon :icon="mdiHandPointingUp" class="mr-2" color="primary"></v-icon>
          Pro Tips: How to Improve Your Score
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col cols="12" md="4">
        <v-card variant="outlined" class="pa-4 h-100 border-success">
          <h3 class="text-subtitle-1 font-weight-bold mb-2 text-success">Lower Your Curve</h3>
          <p class="text-body-2 text-medium-emphasis">
            Cutting high-cost spells for cheaper alternatives (0-2 mana) lowers your <strong>AMV</strong>, which gives you a huge multiplier bonus!
          </p>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card variant="outlined" class="pa-4 h-100 border-info">
          <h3 class="text-subtitle-1 font-weight-bold mb-2 text-info">Add Interaction</h3>
          <p class="text-body-2 text-medium-emphasis">
            Even 1-2 cheap counterspells or removal pieces can push your <strong>Interaction</strong> score higher and protect your game plan.
          </p>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card variant="outlined" class="pa-4 h-100 border-warning">
          <h3 class="text-subtitle-1 font-weight-bold mb-2 text-warning">Focus Your Strategy</h3>
          <p class="text-body-2 text-medium-emphasis">
            Decks that try to do too many things often get a lower score. Pick a clear path (Turbo, Midrange, or Stax) and lean into it!
          </p>
        </v-card>
      </v-col>
    </v-row>

  </v-container>
</template>

<style scoped>
.info-view {
  max-width: 1200px;
  margin: 0 auto;
}

.archetype-card {
  transition: transform 0.2s ease-in-out;
}

.archetype-card:hover {
  transform: translateY(-4px);
  border-color: rgb(var(--v-theme-primary));
}

ul {
  list-style-type: none;
}

li::before {
  content: "•";
  color: rgb(var(--v-theme-primary));
  font-weight: bold;
  display: inline-block;
  width: 1em;
  margin-left: -1em;
}
</style>
