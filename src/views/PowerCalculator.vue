<script setup>
import { ref, onMounted, computed } from "vue";
import { getDecksCommand, getDeckCommand } from "../api/deckCommands.js";
import { evaluateDeckRolesCommand } from "../api/crispiCommands.js";
import DeckTile from "../components/DeckTile.vue";
import { mdiCalculator, mdiDice6, mdiChartBar, mdiChevronLeft } from "@mdi/js";

const decks = ref([]);
const selectedDeck = ref(null);
const isLoadingDecks = ref(false);
const activeTool = ref("crispi"); // Default tool

const crispiResults = ref(null);
const isLoadingCrispi = ref(false);

const sortedRoles = computed(() => {
  if (!crispiResults.value) return [];
  return Object.keys(crispiResults.value.role_counts).sort();
});

onMounted(async () => {
  await loadDecks();
});

async function loadDecks() {
  isLoadingDecks.value = true;
  try {
    decks.value = await getDecksCommand();
  } catch (e) {
    console.error("Failed to load decks", e);
  } finally {
    isLoadingDecks.value = false;
  }
}

async function selectDeck(deckId) {
  try {
    selectedDeck.value = await getDeckCommand(deckId);
    await runCrispi();
  } catch (e) {
    console.error("Failed to select deck", e);
  }
}

function deselectDeck() {
  selectedDeck.value = null;
  crispiResults.value = null;
}

async function runCrispi() {
  if (!selectedDeck.value) return;
  isLoadingCrispi.value = true;
  try {
    crispiResults.value = await evaluateDeckRolesCommand(selectedDeck.value.id);
  } catch (e) {
    console.error("CRISPI evaluation failed", e);
  } finally {
    isLoadingCrispi.value = false;
  }
}

// Hypergeometric Calculator State
const hgPopSize = ref(99);
const hgPopSuccesses = ref(10);
const hgSampleSize = ref(7);
const hgSampleSuccesses = ref(1);
const hgResult = ref(null);

function calculateHypergeometric() {
  const N = hgPopSize.value;
  const K = hgPopSuccesses.value;
  const n = hgSampleSize.value;
  const k = hgSampleSuccesses.value;

  if (k > n || k > K || n > N) {
    hgResult.value = { error: "Invalid parameters" };
    return;
  }

  // P(X = k) = [C(K, k) * C(N-K, n-k)] / C(N, n)
  // We'll calculate P(X >= k) as well.
  
  function combinations(n, k) {
    if (k < 0 || k > n) return 0;
    if (k === 0 || k === n) return 1;
    if (k > n / 2) k = n - k;
    let res = 1;
    for (let i = 1; i <= k; i++) {
      res = res * (n - i + 1) / i;
    }
    return res;
  }

  const probExact = (combinations(K, k) * combinations(N - K, n - k)) / combinations(N, n);
  
  let probAtLeast = 0;
  for (let i = k; i <= Math.min(n, K); i++) {
    probAtLeast += (combinations(K, i) * combinations(N - K, n - i)) / combinations(N, n);
  }

  const probNone = (combinations(K, 0) * combinations(N - K, n)) / combinations(N, n);

  let probLess = 0;
  for (let i = 0; i < k; i++) {
    probLess += (combinations(K, i) * combinations(N - K, n - i)) / combinations(N, n);
  }

  hgResult.value = {
    exact: (probExact * 100).toFixed(2),
    atLeast: (probAtLeast * 100).toFixed(2),
    none: (probNone * 100).toFixed(2),
    lessThan: (probLess * 100).toFixed(2)
  };
}

// Monte Carlo Tool State
const mcSimulations = ref(10000);
const mcResults = ref(null);
const isSimulating = ref(false);

const crispiDimensions = [
  { key: "consistency", label: "Consistency", color: "indigo" },
  { key: "resilience", label: "Resilience", color: "teal" },
  { key: "interaction", label: "Interaction", color: "red" },
  { key: "speed", label: "Speed", color: "amber" },
  { key: "pivotability", label: "Pivotability", color: "deep-purple" }
];

function getInterpretationColor(interpretation) {
  switch (interpretation) {
    case "cEDH Optimized": return "deep-purple-accent-4";
    case "Fringe cEDH": return "purple";
    case "High Power": return "blue";
    case "Focused / Synergistic": return "green";
    default: return "grey";
  }
}

function getTierColor(tier) {
  switch (tier) {
    case "Premium": return "purple-accent-4";
    case "Efficient": return "blue-darken-1";
    case "Slow": return "amber-darken-2";
    default: return "grey";
  }
}

async function runMonteCarlo() {
  if (!selectedDeck.value || !crispiResults.value) return;
  isSimulating.value = true;
  
  // Simulation: Draw 7 cards + 1 card per turn up to turn 10
  // Check probability of having at least 1 card of each role by each turn.
  
  const mainboard = [...selectedDeck.value.cards];
  const commanders = [];
  if (selectedDeck.value.commander) {
    if (selectedDeck.value.commander.Single) {
      commanders.push(selectedDeck.value.commander.Single);
    } else if (selectedDeck.value.commander.Partner) {
      commanders.push(selectedDeck.value.commander.Partner[0]);
      commanders.push(selectedDeck.value.commander.Partner[1]);
    }
  }

  const iterations = mcSimulations.value;
  const maxTurn = 10;
  
  const roleByTurn = {}; // role -> turn -> successes
  const roles = Object.keys(crispiResults.value.role_counts);
  roles.forEach(role => {
    roleByTurn[role] = Array(maxTurn + 1).fill(0);
  });

  // Map card names/ids to their roles for faster lookup
  const cardRolesMap = {};
  crispiResults.value.card_evaluations.forEach(eval_ => {
    cardRolesMap[eval_.card_name] = eval_.roles;
  });

  for (let i = 0; i < iterations; i++) {
    // Shuffle mainboard (Fisher-Yates)
    const shuffled = [...mainboard];
    for (let j = shuffled.length - 1; j > 0; j--) {
      const k = Math.floor(Math.random() * (j + 1));
      [shuffled[j], shuffled[k]] = [shuffled[k], shuffled[j]];
    }
    
    // In Commander, commanders are always "available" (effectively in hand for role purposes)
    const hand = [...commanders];
    
    for (let turn = 1; turn <= maxTurn; turn++) {
      const cardsToDraw = (turn === 1) ? 7 : 1;
      for (let d = 0; d < cardsToDraw; d++) {
        if (shuffled.length > 0) {
          hand.push(shuffled.pop());
        }
      }
      
      // Check roles in hand
      const rolesInHand = new Set();
      hand.forEach(card => {
        const cardRoles = cardRolesMap[card.name] || [];
        cardRoles.forEach(r => rolesInHand.add(r));
      });
      
      rolesInHand.forEach(role => {
        roleByTurn[role][turn]++;
      });
    }
  }

  const processedResults = {};
  roles.forEach(role => {
    processedResults[role] = roleByTurn[role].map(count => (count / iterations * 100).toFixed(1));
  });

  mcResults.value = processedResults;
  isSimulating.value = false;
}

</script>

<template>
  <v-container class="power-calculator-view">
    <div v-if="!selectedDeck">
      <h1 class="text-h4 mb-4 text-primary">Select a Deck to Evaluate</h1>
      <v-divider class="mb-6"></v-divider>
      
      <div v-if="isLoadingDecks" class="d-flex justify-center pa-12">
        <v-progress-circular indeterminate size="64" color="primary"></v-progress-circular>
      </div>
      
      <div v-else class="deck-grid">
        <div 
          v-for="deck in decks" 
          :key="deck.id" 
          @click="selectDeck(deck.id)"
          class="deck-selection-tile"
        >
          <v-card variant="flat" border class="pa-4 h-100 deck-card-hover">
            <div class="d-flex align-center">
              <div>
                <div class="text-h6 font-weight-bold text-left text-primary">{{ deck.name }}</div>
                <div class="text-caption text-medium-emphasis text-left">{{ deck.cards?.length || 0 }} Cards</div>
              </div>
            </div>
          </v-card>
        </div>
      </div>
    </div>

    <div v-else>
      <div class="d-flex align-center gap-4 mb-6">
        <v-btn icon variant="text" @click="deselectDeck" color="primary">
          <v-icon :icon="mdiChevronLeft"></v-icon>
        </v-btn>
        <h1 class="text-h4 text-primary">{{ selectedDeck.name }} Evaluation</h1>
      </div>

      <v-tabs v-model="activeTool" color="primary" grow class="mb-6">
        <v-tab value="crispi">
          <v-icon :icon="mdiChartBar" class="mr-2"></v-icon>
          CRISPI Evaluator
        </v-tab>
        <v-tab value="montecarlo">
          <v-icon :icon="mdiDice6" class="mr-2"></v-icon>
          Monte Carlo
        </v-tab>
        <v-tab value="hypergeometric">
          <v-icon :icon="mdiCalculator" class="mr-2"></v-icon>
          Hypergeometric
        </v-tab>
      </v-tabs>

      <v-window v-model="activeTool">
        <!-- CRISPI Tool -->
        <v-window-item value="crispi">
          <v-row v-if="crispiResults">
            <!-- Summary Card -->
            <v-col cols="12">
              <v-card variant="flat" border class="pa-6 mb-4 text-center">
                <div class="text-overline mb-1">Total CRISPI Score</div>
                <div class="d-flex align-center justify-center mb-2">
                  <div class="text-h2 font-weight-black mr-4">{{ crispiResults.crispi.total_score.toFixed(1) }}</div>
                  <div class="text-h4 text-primary font-weight-bold">/ 25</div>
                </div>
                <div class="text-caption text-medium-emphasis mb-2">
                  Raw Score: {{ crispiResults.crispi.raw_score.toFixed(1) }} | 
                  AMV Multiplier: {{ crispiResults.crispi.amv_multiplier.toFixed(2) }}x
                  <span v-if="crispiResults.crispi.combo_multiplier > 1.0"> & Combo: {{ crispiResults.crispi.combo_multiplier.toFixed(2) }}x </span>
                  | Final Multiplier: {{ crispiResults.crispi.final_multiplier.toFixed(2) }}x |
                  Bracket: {{ crispiResults.crispi.bracket }} |
                  Archetype: {{ crispiResults.crispi.archetype }}
                </div>

                <!-- Structural Metrics Row -->
                <div class="d-flex justify-center flex-wrap gap-4 mb-4">
                  <v-chip size="small" variant="outlined" color="primary">
                    Role Score: {{ crispiResults.crispi.role_score.toFixed(2) }}
                  </v-chip>
                  <v-chip size="small" variant="outlined" color="primary">
                    Land Score: {{ crispiResults.crispi.land_score.toFixed(2) }}
                  </v-chip>
                  <v-chip size="small" variant="outlined" :color="crispiResults.crispi.commander_mv_penalty > 0 ? 'warning' : 'primary'">
                    Cmdr Penalty: -{{ crispiResults.crispi.commander_mv_penalty.toFixed(2) }}
                  </v-chip>
                </div>
                
                <v-alert
                  v-if="crispiResults.crispi.detected_combos.length > 0"
                  density="compact"
                  variant="tonal"
                  color="error"
                  class="mt-4 text-left"
                >
                  <div class="text-subtitle-2 font-weight-bold mb-1">Detected Infinite Combos:</div>
                  <ul class="pl-4">
                    <li v-for="combo in crispiResults.crispi.detected_combos" :key="combo" class="text-caption">
                      {{ combo }}
                    </li>
                  </ul>
                </v-alert>
                <v-chip
                  :color="getInterpretationColor(crispiResults.crispi.interpretation)"
                  size="large"
                  label
                  class="text-uppercase font-weight-bold"
                >
                  {{ crispiResults.crispi.interpretation }}
                </v-chip>
              </v-card>
            </v-col>

            <!-- Dimension Breakdown -->
            <v-col v-for="dim in crispiDimensions" :key="dim.key" cols="12" md="6" lg="4">
              <v-card variant="flat" border class="pa-4 h-100">
                <div class="d-flex justify-space-between align-center mb-4">
                  <div class="d-flex align-center">
                    <v-avatar :color="dim.color" size="32" class="mr-3 text-white">
                      {{ dim.label[0] }}
                    </v-avatar>
                    <h3 class="text-h6">{{ dim.label }}</h3>
                  </div>
                  <div class="text-h5 font-weight-bold">{{ crispiResults.crispi[dim.key].score }} <span class="text-caption text-medium-emphasis">/ 5</span></div>
                </div>
                
                <v-progress-linear
                  :model-value="crispiResults.crispi[dim.key].score"
                  :max="5"
                  :color="dim.color"
                  rounded
                  height="8"
                  class="mb-4"
                ></v-progress-linear>
              </v-card>
            </v-col>

            <!-- Original Role Distribution -->
            <v-col cols="12">
              <v-divider class="my-6"></v-divider>
              <h3 class="text-h5 mb-4">Detailed Role Distribution</h3>
            </v-col>
            <v-col cols="12" md="4">
              <v-card variant="flat" border class="pa-4">
                <h3 class="text-h6 mb-4">Role Distribution</h3>
                <div v-if="isLoadingCrispi" class="d-flex justify-center pa-6">
                  <v-progress-circular indeterminate></v-progress-circular>
                </div>
                <div v-else-if="crispiResults">
                  <div v-for="role in sortedRoles" :key="role" class="mb-3">
                    <div class="d-flex justify-space-between align-center mb-1">
                      <span class="text-subtitle-2">{{ role }}</span>
                      <span class="font-weight-bold">{{ crispiResults.role_counts[role].toFixed(1) }}</span>
                    </div>
                    <v-progress-linear 
                      :model-value="crispiResults.role_counts[role]" 
                      :max="selectedDeck.cards.length" 
                      color="primary" 
                      rounded 
                      height="8"
                    ></v-progress-linear>
                  </div>
                </div>
              </v-card>
            </v-col>
            <v-col cols="12" md="8">
              <v-card variant="flat" border class="pa-4">
                <h3 class="text-h6 mb-4">Card Categorization</h3>
                <v-expansion-panels variant="accordion">
                  <v-expansion-panel
                    v-for="role in sortedRoles"
                    :key="role"
                    :title="`${role} (${(crispiResults?.role_counts[role] || 0).toFixed(1)})`"
                  >
                    <v-expansion-panel-text>
                      <v-list density="compact">
                        <v-list-item
                          v-for="eval_ in crispiResults.card_evaluations.filter(e => e.roles.includes(role))"
                          :key="eval_.card_id"
                          :title="eval_.card_name"
                        >
                          <template v-slot:append>
                            <v-chip size="x-small" :color="getTierColor(eval_.tier)" label>{{ eval_.tier }}</v-chip>
                          </template>
                        </v-list-item>
                      </v-list>
                    </v-expansion-panel-text>
                  </v-expansion-panel>
                </v-expansion-panels>
              </v-card>
            </v-col>
          </v-row>
        </v-window-item>

        <!-- Monte Carlo Tool -->
        <v-window-item value="montecarlo">
          <v-card variant="flat" border class="pa-4">
            <div class="d-flex justify-space-between align-center mb-6">
              <div>
                <h3 class="text-h6">Draw Consistency Simulation</h3>
                <p class="text-body-2 text-medium-emphasis">Probability of having at least one card of a role by turn X.</p>
              </div>
              <v-btn color="primary" @click="runMonteCarlo" :loading="isSimulating">
                Run Simulation
              </v-btn>
            </div>

            <div v-if="mcResults" class="mc-results-table overflow-x-auto">
              <table class="w-100">
                <thead>
                  <tr>
                    <th class="text-left pa-2">Role</th>
                    <th v-for="t in 10" :key="t" class="text-center pa-2">T{{ t }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(probs, role) in mcResults" :key="role">
                    <td class="text-left pa-2 font-weight-medium">{{ role }}</td>
                    <td v-for="t in 10" :key="t" class="text-center pa-2">
                      {{ probs[t] }}%
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-else class="text-center pa-12 text-medium-emphasis">
              Click "Run Simulation" to see draw probabilities based on current CRISPI roles.
            </div>
          </v-card>
        </v-window-item>

        <!-- Hypergeometric Tool -->
        <v-window-item value="hypergeometric">
          <v-row>
            <v-col cols="12" md="4">
              <v-card variant="flat" border class="pa-4">
                <h3 class="text-h6 mb-4">Parameters</h3>
                <v-text-field v-model.number="hgPopSize" label="Population Size" persistent-hint hint="Cards in your deck / library you are drawing from" type="number" density="compact" class="mb-4"></v-text-field>
                <v-text-field v-model.number="hgPopSuccesses" label="Successes in population" persistent-hint hint="Number of cards you want that is in the deck / library" type="number" density="compact" class="mb-4"></v-text-field>
                <v-text-field v-model.number="hgSampleSize" label="Sample Size" persistent-hint hint="Number of cards we are drawing ex. cards in opening hand" type="number" density="compact" class="mb-4"></v-text-field>
                <v-text-field v-model.number="hgSampleSuccesses" label="Successes in Sample" persistent-hint hint="Number of wanted cards you want to draw" type="number" density="compact" class="mb-4"></v-text-field>
                <v-btn color="primary" block @click="calculateHypergeometric" class="mt-2">Calculate</v-btn>
              </v-card>
            </v-col>
            <v-col cols="12" md="8">
              <v-card variant="flat" border class="pa-4 h-100 d-flex flex-column justify-center align-center">
                <div v-if="hgResult">
                  <div v-if="hgResult.error" class="text-error">{{ hgResult.error }}</div>
                  <template v-else>
                    <v-row>
                      <v-col cols="6" class="text-center mb-6">
                        <div class="text-subtitle-1 text-medium-emphasis">Probability of EXACTLY {{ hgSampleSuccesses }}</div>
                        <div class="text-h4 font-weight-bold color-primary text-primary">{{ hgResult.exact }}%</div>
                      </v-col>
                      <v-col cols="6" class="text-center mb-6">
                        <div class="text-subtitle-1 text-medium-emphasis">Probability of AT LEAST {{ hgSampleSuccesses }}</div>
                        <div class="text-h4 font-weight-bold text-success">{{ hgResult.atLeast }}%</div>
                      </v-col>
                      <v-col cols="6" class="text-center">
                        <div class="text-subtitle-1 text-medium-emphasis">Probability of NONE</div>
                        <div class="text-h4 font-weight-bold text-error">{{ hgResult.none }}%</div>
                      </v-col>
                      <v-col cols="6" class="text-center">
                        <div class="text-subtitle-1 text-medium-emphasis">Probability of LESS THAN {{ hgSampleSuccesses }}</div>
                        <div class="text-h4 font-weight-bold text-orange-darken-2">{{ hgResult.lessThan }}%</div>
                      </v-col>
                    </v-row>
                  </template>
                </div>
                <div v-else class="text-medium-emphasis">
                  Enter parameters to calculate probabilities.
                </div>
              </v-card>
            </v-col>
          </v-row>
        </v-window-item>
      </v-window>
    </div>
  </v-container>
</template>

<style scoped>
.deck-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.deck-selection-tile {
  cursor: pointer;
  transition: transform 0.2s;
}

.deck-selection-tile:hover {
  transform: translateY(-4px);
}

.deck-card-hover:hover {
  border-color: rgb(var(--v-theme-primary)) !important;
  background-color: rgba(var(--v-theme-primary), 0.02) !important;
}

.gap-4 {
  gap: 16px;
}

.w-100 {
  width: 100%;
}

.mc-results-table table {
  border-collapse: collapse;
}

.mc-results-table th {
  border-bottom: 2px solid rgba(0,0,0,0.1);
}

.mc-results-table tr:nth-child(even) {
  background-color: rgba(0,0,0,0.02);
}

.mc-results-table td, .mc-results-table th {
  min-width: 60px;
}
</style>
