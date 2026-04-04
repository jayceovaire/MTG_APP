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
} from "@mdi/js";

const archetypes = [
  {
    title: "Turbo",
    icon: mdiLightningBolt,
    color: "amber",
    definition: "A deck that aims to win as quickly as possible, leveraging fast mana, efficient win conditions, and explosive draw engines.",
    traits: [
      "High number of fast mana sources (Mana Rocks, Rituals, Treasures)",
      "Low average mana value for spells",
      "Premium tutors and burst/mass/wheel draw",
      "Win condition is usually achieved by turn 3-5",
    ],
    strategy: "Go fast, win before opponents stabilize.",
  },
  {
    title: "Midrange",
    icon: mdiSwapHorizontal,
    color: "blue",
    definition: "A deck that balances early interaction with efficient threats, aiming to outvalue opponents over the midgame.",
    traits: [
      "Moderate-speed spells with flexible utility",
      "Strong consistency and resilience (tutors, draw engines, recursion)",
      "Relies on board presence and incremental advantage",
    ],
    strategy: "Play strong cards in each stage of the game, maintain flexibility.",
  },
  {
    title: "Stax",
    icon: mdiShieldLock,
    color: "deep-orange",
    definition: "A deck built around denying resources and locking opponents out of the game.",
    traits: [
      "Disruptive effects targeting opponent lands, mana, or spell casting",
      "Spells often impose costs or restrictions",
      "Slower game plan, focusing on long-term control",
    ],
    strategy: "Lock opponents while advancing your game plan.",
  },
  {
    title: "Commander Engine",
    icon: mdiCogs,
    color: "purple",
    definition: "A deck relying on repeatable value engines, often centralized around the commander or other key pieces.",
    traits: [
      "Commander or mainboard engines generate card advantage every turn",
      "Tutors, draw engines, recursion, or ramp engines are present",
      "Typically combines high consistency with mid-to-high speed",
    ],
    strategy: "Use engines to grind value and achieve long-term inevitability.",
  },
];

const dimensions = [
  { name: "Consistency (C)", icon: mdiBullseyeArrow, desc: "Tutors, draw engines, and engine pieces contribute. Weighted by card quality. High win condition density also rewards consistency." },
  { name: "Resilience (R)", icon: mdiShieldCheck, desc: "Protection and recursion effects, survival-focused tools. Capped if the deck cannot function without its commander." },
  { name: "Interaction (I)", icon: mdiGestureTap, desc: "Removal, stax, and free interaction contribute. Requires at least 6 pieces of ≤2 MV instant-speed interaction for scores ≥3." },
  { name: "Speed (S)", icon: mdiSpeedometer, desc: "Evaluated on both win turn (Efficiency) and the ability to generate mana/draw bursts (Explosiveness/Velocity)." },
  { name: "Pivotability (P)", icon: mdiScaleBalance, desc: "Versatility, multi-role cards, and flexible responses. Rewards cards that serve multiple win lines." },
];

const structuralMetrics = [
  { name: "Role Score", desc: "A normalized measure of the deck's raw role coverage (Raw Score / 25.0)." },
  { name: "Land Score", desc: "Measures the adequacy of the land count, normalized against a standard 38-land baseline." },
  { name: "Commander MV Penalty", desc: "A penalty applied based on the total mana value of your commanders, reflecting the cost of accessing the command zone." },
];

const brackets = [
  { score: "≥ 23.0", tier: "cEDH Optimized", desc: "Exceptionally fast and efficient, top-tier competitive decks." },
  { score: "≥ 19.0", tier: "Fringe cEDH", desc: "Competitive with high synergy, slightly slower or less consistent." },
  { score: "≥ 15.0", tier: "High Power", desc: "Strong decks with clear win strategies, viable in most environments." },
  { score: "≥ 9.0", tier: "Focused / Synergistic", desc: "Good decks, some inconsistency or slower interaction." },
  { score: "< 9.0", tier: "Casual / Battlecruiser", desc: "Fun or experimental decks, often lacking efficiency or synergy." },
];

const amvMultipliers = [
  { amv: "≤ 1.3", mult: "1.24" },
  { amv: "≤ 1.6", mult: "1.16" },
  { amv: "≤ 2.0", mult: "1.06" },
  { amv: "≤ 2.4", mult: "1.02" },
  { amv: "≤ 2.8", mult: "1.00" },
  { amv: "≤ 3.0", mult: "0.94" },
  { amv: "≤ 3.4", mult: "0.88" },
  { amv: "≤ 3.8", mult: "0.72" },
  { amv: "> 3.8", mult: "0.55" },
];
</script>

<template>
  <v-container class="info-view pb-12">
    <v-row>
      <v-col cols="12">
        <h1 class="text-h3 font-weight-bold mb-2 text-primary">Crispi Evaluation</h1>
        <p class="text-subtitle-1 text-medium-emphasis mb-8">
          A comprehensive framework for analyzing Magic: The Gathering decks based on strategy, speed, and efficiency.
        </p>
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
        <v-card variant="flat" border class="h-100 text-center py-4 px-2">
          <v-icon :icon="dim.icon" size="x-large" color="primary" class="mb-2"></v-icon>
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
          Structural Floors & Overrides
        </h2>
        <v-divider class="mb-6"></v-divider>
      </v-col>
      <v-col cols="12">
        <v-card variant="outlined" class="pa-4">
          <p class="mb-4 text-body-1">
            CRISPI applies "Structural Floors" to ensure decks with specific powerful attributes meet a minimum score in relevant categories.
          </p>
          <v-table density="compact">
            <thead>
              <tr>
                <th class="text-left">Condition</th>
                <th class="text-left">Minimum Score (Floor)</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>Commander is a card advantage engine</td>
                <td>Consistency ≥ 4, Pivotability ≥ 4</td>
              </tr>
              <tr>
                <td>8+ free interaction spells</td>
                <td>Interaction = 5, Resilience ≥ 4</td>
              </tr>
              <tr>
                <td>Compact ≤2 card deterministic win package</td>
                <td>Speed = 5, Consistency +1</td>
              </tr>
              <tr>
                <td>10+ fast mana pieces</td>
                <td>Speed = 5</td>
              </tr>
              <tr>
                <td>5+ premium ≤2 MV tutors</td>
                <td>Consistency = 5</td>
              </tr>
            </tbody>
          </v-table>
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
            CRISPI automatically detects two-card infinite combos in your deck. Finding a combo significantly increases your deck's power score and bracket.
          </p>
          <v-row>
            <v-col cols="12" md="6">
              <h3 class="text-h6 mb-2">Multiplier Rules</h3>
              <v-list density="compact">
                <v-list-item>
                  <v-list-item-title>Max Bonus (per combo): <span class="text-primary font-weight-bold">+0.20x</span></v-list-item-title>
                  <v-list-item-subtitle>For the fastest combos (total MV ≤ 3)</v-list-item-subtitle>
                </v-list-item>
                <v-list-item>
                  <v-list-item-title>MV & Speed Scaling</v-list-item-title>
                  <v-list-item-subtitle>Multipliers decrease for expensive or slow (Sorcery/Tap) combos</v-list-item-subtitle>
                </v-list-item>
                <v-list-item>
                  <v-list-item-title>Tutor Bonus: <span class="text-primary font-weight-bold">+0.05x</span></v-list-item-title>
                  <v-list-item-subtitle>Applied if the deck contains at least one tutor</v-list-item-subtitle>
                </v-list-item>
              </v-list>
            </v-col>
            <v-col cols="12" md="6">
              <h3 class="text-h6 mb-2">Bracket & Score Limits</h3>
              <v-alert type="info" variant="tonal" border="start" density="comfortable" class="mb-4">
                Detecting a valid two-card infinite combo automatically sets your deck to <strong>Bracket 4</strong> or higher.
              </v-alert>
              <div class="text-caption text-medium-emphasis">
                * Note: Total Combo Multiplier is capped at 1.30x.
              </div>
              <div class="text-caption text-medium-emphasis">
                * Note: Final score is capped at 25.0 to maintain a consistent power scale.
              </div>
            </v-col>
          </v-row>
        </v-card>
      </v-col>
    </v-row>

    <!-- Power Ratings & AMV -->
    <v-row>
      <v-col cols="12" md="7">
        <v-card variant="outlined">
          <v-card-title class="bg-surface-variant py-3">Power Tier Brackets</v-card-title>
          <v-table density="comfortable">
            <thead>
              <tr>
                <th class="text-left">Total Score</th>
                <th class="text-left">Power Tier</th>
                <th class="text-left">Description</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in brackets" :key="item.tier">
                <td class="font-weight-bold">{{ item.score }}</td>
                <td>
                  <v-chip size="small" variant="tonal" color="primary">{{ item.tier }}</v-chip>
                </td>
                <td class="text-caption text-medium-emphasis">{{ item.desc }}</td>
              </tr>
            </tbody>
          </v-table>
        </v-card>
      </v-col>

      <v-col cols="12" md="5">
        <v-card variant="outlined">
          <v-card-title class="bg-surface-variant py-3">AMV Multiplier</v-card-title>
          <v-card-text class="pt-4">
            <p class="text-body-2 mb-4">
              The <strong>Average Mana Value (AMV)</strong> reflects your deck's efficiency. Lower AMV decks receive a bonus to their score.
            </p>
            <v-table density="comfortable" class="border rounded">
              <thead>
                <tr>
                  <th class="text-left">AMV</th>
                  <th class="text-left">Multiplier</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in amvMultipliers" :key="row.amv">
                  <td>{{ row.amv }}</td>
                  <td class="font-weight-bold">{{ row.mult }}x</td>
                </tr>
              </tbody>
            </v-table>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Non-technical explanation -->
    <v-row class="mt-8">
      <v-col cols="12">
        <v-alert color="primary" variant="tonal" border="start">
          <h3 class="text-h6 mb-2">How it works</h3>
          <p class="text-body-2">
            The app analyzes every card in your deck and assigns them <strong>Roles</strong> (like Ramp, Draw, or Removal). 
            Based on the density and quality of these roles, it calculates a <strong>Raw Score</strong> across five dimensions. 
            Archetype-specific adjustments and structural floors are then applied to reflect competitive viability.
            Finally, it applies a combined multiplier from your <strong>AMV</strong> and any <strong>Infinite Combos</strong> 
            to determine your <strong>Total Score</strong> (capped at 25.0).
          </p>
        </v-alert>
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
