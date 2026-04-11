<script setup>
import { ref, onMounted, computed, watch } from "vue";
import { getDecksCommand, getDeckCommand } from "../api/deckCommands.js";
import { evaluateDeckRolesCommand } from "../api/crispiCommands.js";
import DeckTile from "../components/DeckTile.vue";
import { mdiCalculator, mdiDice6, mdiChartBar, mdiChevronLeft, mdiVectorPolyline } from "@mdi/js";

const decks = ref([]);
const selectedDeck = ref(null);
const pendingDeckName = ref("");
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
  const deckMeta = decks.value.find(deck => deck.id === deckId);
  pendingDeckName.value = deckMeta?.name || "";
  crispiResults.value = null;
  isLoadingCrispi.value = true;
  try {
    selectedDeck.value = await getDeckCommand(deckId);
    await runCrispi();
  } catch (e) {
    console.error("Failed to select deck", e);
    selectedDeck.value = null;
    crispiResults.value = null;
  } finally {
    pendingDeckName.value = "";
  }
}

function deselectDeck() {
  selectedDeck.value = null;
  crispiResults.value = null;
  pendingDeckName.value = "";
}

async function runCrispi() {
  if (!selectedDeck.value) return;
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
const hgTargetTurn = ref(1);
const hgResult = ref(null);
const hgCardType = ref("Any");
const hgCrispiRole = ref("Any");

function formatRole(role) {
  if (!role) return "";
  return role.replace(/_/g, " ");
}

const cardTypes = ["Any", "Creature", "Instant", "Sorcery", "Enchantment", "Artifact", "Land", "Planeswalker", "Battle", "Tribal"];
const crispiRoles = computed(() => {
  if (!crispiResults.value) return ["Any"];
  const allRoles = new Set();
  crispiResults.value.card_evaluations.forEach(card => {
    card.roles.forEach(role => allRoles.add(role));
  });
  return ["Any", ...Array.from(allRoles).sort()].map(role => ({ title: formatRole(role), value: role }));
});

watch(hgTargetTurn, (turn) => {
  hgSampleSize.value = 7 + (turn - 1);
});

watch([hgCardType, hgCrispiRole, crispiResults], () => {
  if (!crispiResults.value) return;

  const libraryCards = crispiResults.value.card_evaluations.filter(c => !c.is_commander);
  hgPopSize.value = libraryCards.length;

  const matches = libraryCards.filter(card => {
    const typeMatch = hgCardType.value === "Any" || card.card_types.includes(hgCardType.value);
    const roleMatch = hgCrispiRole.value === "Any" || card.roles.includes(hgCrispiRole.value);
    return typeMatch && roleMatch;
  });

  hgPopSuccesses.value = matches.length;
  
  if (hgResult.value) {
    calculateHypergeometric();
  }
}, { immediate: true });

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
const mcTargetCount = ref(1);
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

function getIntegrationColor(score) {
  if (score >= 0.7) return "deep-purple-accent-4";
  if (score >= 0.4) return "primary";
  if (score >= 0.15) return "amber-darken-2";
  return "red-darken-2";
}

function getMcColor(prob) {
  if (prob === undefined || prob === null) return '';
  const p = parseFloat(prob);
  if (p > 75) return 'bg-green-lighten-4 text-green-darken-4 font-weight-bold';
  if (p <= 50) return 'bg-red-lighten-4 text-red-darken-4';
  if (p > 50 && p <= 75) return 'bg-amber-lighten-4 text-amber-darken-4';
  return '';
}

async function runMonteCarlo() {
  if (!selectedDeck.value || !crispiResults.value) return;
  isSimulating.value = true;
  
  // Simulation: Draw 7 cards + 1 card per turn up to turn 10
  // Check probability of having at least 1 card of each role by each turn.
  
  const mainboard = [...selectedDeck.value.cards];
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
    
    const hand = [];
    
    for (let turn = 1; turn <= maxTurn; turn++) {
      const cardsToDraw = (turn === 1) ? 7 : 1;
      for (let d = 0; d < cardsToDraw; d++) {
        if (shuffled.length > 0) {
          hand.push(shuffled.pop());
        }
      }
      
      // Check roles in hand
      const roleCountsInHand = {};
      hand.forEach(card => {
        const cardRoles = cardRolesMap[card.name] || [];
        cardRoles.forEach(r => {
          roleCountsInHand[r] = (roleCountsInHand[r] || 0) + 1;
        });
      });
      
      roles.forEach(role => {
        if ((roleCountsInHand[role] || 0) >= mcTargetCount.value) {
          roleByTurn[role][turn]++;
        }
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
        <h1 class="text-h4 text-primary">{{ selectedDeck.name || pendingDeckName }} Evaluation</h1>
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
        <v-tab value="integration">
          <v-icon :icon="mdiVectorPolyline" class="mr-2"></v-icon>
          Integration
        </v-tab>
      </v-tabs>

      <v-window v-model="activeTool">
        <!-- CRISPI Tool -->
        <v-window-item value="crispi">
          <v-row v-if="isLoadingCrispi && !crispiResults">
            <v-col cols="12">
              <v-card variant="flat" border class="pa-6 mb-4">
                <div class="d-flex align-center justify-space-between mb-4">
                  <v-skeleton-loader type="heading" class="flex-grow-1 mr-4"></v-skeleton-loader>
                  <v-skeleton-loader type="chip"></v-skeleton-loader>
                </div>
                <v-skeleton-loader type="paragraph"></v-skeleton-loader>
                <div class="d-flex flex-wrap gap-4 mt-4">
                  <v-skeleton-loader v-for="n in 4" :key="n" type="chip"></v-skeleton-loader>
                </div>
              </v-card>
            </v-col>
            <v-col v-for="n in 5" :key="`dim-${n}`" cols="12" md="6" lg="4">
              <v-card variant="flat" border class="pa-4 h-100">
                <v-skeleton-loader type="heading, paragraph"></v-skeleton-loader>
              </v-card>
            </v-col>
            <v-col cols="12" md="4">
              <v-card variant="flat" border class="pa-4">
                <v-skeleton-loader type="heading, list-item-three-line, list-item-three-line, list-item-three-line"></v-skeleton-loader>
              </v-card>
            </v-col>
            <v-col cols="12" md="8">
              <v-card variant="flat" border class="pa-4">
                <v-skeleton-loader type="heading, list-item-two-line, list-item-two-line, list-item-two-line, list-item-two-line"></v-skeleton-loader>
              </v-card>
            </v-col>
          </v-row>
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
                <div class="d-flex justify-center flex-wrap gap-4 mb-2">
                  <v-chip size="small" variant="outlined" :color="crispiResults.crispi.role_score > 1.0 ? 'success' : crispiResults.crispi.role_score < 1.0 ? 'warning' : 'primary'">
                    Role Score: {{ crispiResults.crispi.role_score.toFixed(2) }}
                  </v-chip>
                  <v-chip size="small" variant="outlined" :color="crispiResults.crispi.land_score > 1.0 ? 'warning' : crispiResults.crispi.land_score < 1.0 ? 'error' : 'success'">
                    Land Score: {{ crispiResults.crispi.land_score.toFixed(2) }}
                  </v-chip>
                  <v-chip size="small" variant="outlined" :color="crispiResults.crispi.commander_mv_penalty > 0 ? 'warning' : 'success'">
                    Cmdr {{ crispiResults.crispi.commander_mv_penalty > 0 ? 'Penalty' : 'Bonus' }}: 
                    {{ crispiResults.crispi.commander_mv_penalty > 0 ? '-' : '+' }}{{ Math.abs(crispiResults.crispi.commander_mv_penalty).toFixed(2) }}
                  </v-chip>
                </div>

                <!-- Archetype Signals Row -->
                <div class="d-flex justify-center flex-wrap gap-2 mb-4">
                  <v-chip size="x-small" variant="tonal" color="amber" v-if="crispiResults.crispi.turbo_signal >= 1.0">
                    Turbo: {{ crispiResults.crispi.turbo_signal.toFixed(1) }}
                  </v-chip>
                  <v-chip size="x-small" variant="tonal" color="blue" v-if="crispiResults.crispi.midrange_signal >= 1.0">
                    Midrange: {{ crispiResults.crispi.midrange_signal.toFixed(1) }}
                  </v-chip>
                  <v-chip size="x-small" variant="tonal" color="deep-orange" v-if="crispiResults.crispi.stax_signal >= 1.0">
                    Stax: {{ crispiResults.crispi.stax_signal.toFixed(1) }}
                  </v-chip>
                  <v-chip size="x-small" variant="tonal" color="green" v-if="crispiResults.crispi.voltron_signal >= 1.0">
                    Voltron: {{ crispiResults.crispi.voltron_signal.toFixed(1) }}
                  </v-chip>
                  <v-chip size="x-small" variant="tonal" color="pink" v-if="crispiResults.crispi.group_hug_signal >= 1.0">
                    Group Hug: {{ crispiResults.crispi.group_hug_signal.toFixed(1) }}
                  </v-chip>
                  <v-chip size="x-small" variant="tonal" color="light-green-darken-4" v-if="crispiResults.crispi.infect_signal >= 1.0">
                    Infect: {{ crispiResults.crispi.infect_signal.toFixed(1) }}
                  </v-chip>
                  <v-chip size="x-small" variant="tonal" color="purple" v-if="crispiResults.crispi.commander_engine_signal > 0">
                    CmdrEngine: {{ crispiResults.crispi.commander_engine_signal.toFixed(1) }}
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
                      <span class="text-subtitle-2">{{ formatRole(role) }}</span>
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
                    :title="`${formatRole(role)} (${(crispiResults?.role_counts[role] || 0).toFixed(1)})`"
                  >
                    <v-expansion-panel-text>
                      <v-list density="compact">
                        <v-list-item
                          v-for="eval_ in crispiResults.card_evaluations.filter(e => e.roles.includes(role))"
                          :key="eval_.card_id"
                          :title="eval_.card_name"
                        >
                          <template v-slot:append>
                            <v-chip size="x-small" :color="getTierColor(eval_.tier)" label class="mr-1">{{ eval_.tier }}</v-chip>
                            <v-chip size="x-small" :color="getIntegrationColor(eval_.integration)" label class="text-white">{{ (eval_.integration * 100).toFixed(0) }}%</v-chip>
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
            <div class="d-flex justify-space-between align-end mb-6">
              <div>
                <h3 class="text-h6">Draw Consistency Simulation</h3>
                <p class="text-body-2 text-medium-emphasis">Probability of having at least <strong>{{ mcTargetCount }}</strong> card(s) of a role by turn X.</p>
              </div>
              <div class="d-flex align-center gap-4">
                <v-text-field
                  v-model.number="mcTargetCount"
                  label="Target Count"
                  type="number"
                  density="compact"
                  variant="outlined"
                  min="1"
                  max="10"
                  hide-details
                  style="width: 120px"
                ></v-text-field>
                <v-btn color="primary" @click="runMonteCarlo" :loading="isSimulating" height="40">
                  Run Simulation
                </v-btn>
              </div>
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
                    <td class="text-left pa-2 font-weight-medium">{{ formatRole(role) }}</td>
                    <td v-for="t in 10" :key="t" :class="['text-center pa-2', getMcColor(probs[t])]">
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
                <h3 class="text-h6 mb-4">Selection Criteria</h3>
                <v-select
                  v-model="hgCardType"
                  :items="cardTypes"
                  label="Card Type"
                  density="compact"
                  class="mb-2"
                  variant="outlined"
                ></v-select>
                <v-select
                  v-model="hgCrispiRole"
                  :items="crispiRoles"
                  item-title="title"
                  item-value="value"
                  label="CRISPI Role"
                  density="compact"
                  class="mb-4"
                  variant="outlined"
                ></v-select>

                <h3 class="text-h6 mb-4">Parameters</h3>
                <v-text-field v-model.number="hgTargetTurn" label="Target Turn" persistent-hint hint="Turn for sample size (Turn 1 = 7 cards)" type="number" density="compact" class="mb-4" min="1"></v-text-field>
                <v-text-field v-model.number="hgPopSize" label="Population Size (N)" persistent-hint hint="Total cards in library" type="number" density="compact" class="mb-4"></v-text-field>
                <v-text-field v-model.number="hgPopSuccesses" label="Successes in population (K)" persistent-hint hint="Matches for selected criteria" type="number" density="compact" class="mb-4"></v-text-field>
                <v-text-field v-model.number="hgSampleSize" label="Sample Size (n)" persistent-hint hint="Total cards to draw" type="number" density="compact" class="mb-4"></v-text-field>
                <v-text-field v-model.number="hgSampleSuccesses" label="Successes in Sample (k)" persistent-hint hint="Wanted number of cards to draw" type="number" density="compact" class="mb-4"></v-text-field>
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

        <!-- Integration Tool -->
        <v-window-item value="integration">
          <v-card variant="flat" border class="pa-4">
            <div class="mb-6">
              <h3 class="text-h6">Card Integration Network</h3>
              <p class="text-body-2 text-medium-emphasis">
                Integration measures how many deck relationships (tutors, wincon groups, engine tags, type dependencies) point to a card.
                Higher integration indicates core pieces, while lowest values may reveal "dead cards."
              </p>
            </div>

            <v-table density="compact" v-if="crispiResults">
              <thead>
                <tr>
                  <th class="text-left">Card Name</th>
                  <th class="text-center">Roles</th>
                  <th class="text-center" style="width: 200px">Integration Graph</th>
                  <th class="text-right" style="width: 100px">Weight</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="eval_ in [...crispiResults.card_evaluations].sort((a, b) => b.integration - a.integration)" :key="eval_.card_id">
                  <td class="font-weight-bold">{{ eval_.card_name }}</td>
                  <td class="text-center">
                    <div class="d-flex flex-wrap gap-1 justify-center">
                      <v-chip 
                        v-for="role in eval_.roles" 
                        :key="role" 
                        size="x-small" 
                        variant="tonal" 
                        class="text-caption"
                      >
                        {{ formatRole(role) }}
                      </v-chip>
                    </div>
                  </td>
                  <td>
                    <v-progress-linear
                      :model-value="eval_.integration * 100"
                      :color="getIntegrationColor(eval_.integration)"
                      height="12"
                      rounded
                    >
                      <template v-slot:default="{ value }">
                        <span class="text-caption text-white font-weight-bold" style="font-size: 8px !important;">{{ value.toFixed(0) }}%</span>
                      </template>
                    </v-progress-linear>
                  </td>
                  <td class="text-right">
                    <v-chip 
                      size="x-small" 
                      :color="getIntegrationColor(eval_.integration)" 
                      label 
                      variant="flat"
                      class="text-white"
                    >
                      {{ (eval_.integration * 100).toFixed(1) }}
                    </v-chip>
                  </td>
                </tr>
              </tbody>
            </v-table>
          </v-card>
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
  border: 1px solid rgba(0,0,0,0.05);
}
</style>
