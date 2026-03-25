<script setup>
import { computed, onMounted, ref, watch } from "vue";
import { mdiAlertCircleOutline, mdiCardsOutline, mdiCrownOutline, mdiPlus, mdiRefresh } from "@mdi/js";
import {
  addCardToDeckCommand,
  deleteDeckCommanderCommand,
  getDeckCommand,
  removeDeckCommanderCommand,
  removeCardFromDeckCommand,
  setDeckCommanderCommand,
} from "../api/deckCommands.js";
import DeckCardRow from "../components/DeckCardRow.vue";

const props = defineProps({
  deckId: {
    type: [String, Number],
    required: true,
  },
});

const deck = ref(null);
const isLoading = ref(false);
const isUpdatingDeck = ref(false);
const loadError = ref("");
const searchName = ref("");
const isAddingCard = ref(false);
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");

const typeDisplayOrder = [
  "Creature",
  "Artifact",
  "Enchantment",
  "Instant",
  "Sorcery",
  "Planeswalker",
  "Battle",
  "Land",
  "Other",
];

function normalizeDeckId(value) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

async function loadDeck() {
  isLoading.value = true;
  loadError.value = "";

  try {
    deck.value = await getDeckCommand(normalizeDeckId(props.deckId));
  } catch (e) {
    deck.value = null;
    loadError.value = `Failed to load deck: ${String(e)}`;
    console.error(e);
  } finally {
    isLoading.value = false;
  }
}

function showSuccess(message) {
  snackbarMessage.value = message;
  snackbarColor.value = "success";
  snackbarVisible.value = true;
}

function showError(message) {
  snackbarMessage.value = message;
  snackbarColor.value = "error";
  snackbarVisible.value = true;
}

function commanderCards(commander) {
  if (!commander) {
    return [];
  }

  if (typeof commander === "string") {
    return commander === "None" ? [] : [];
  }

  if (commander.Single) {
    return [commander.Single];
  }

  if (Array.isArray(commander.Partner)) {
    return commander.Partner;
  }

  return [];
}

function primaryType(card) {
  const types = Array.isArray(card.card_type) ? card.card_type : [];
  return typeDisplayOrder.find((type) => types.includes(type)) || "Other";
}

function canSetCommander(card) {
  const types = Array.isArray(card?.card_type) ? card.card_type : [];
  const superTypes = Array.isArray(card?.super_type) ? card.super_type : [];
  const subTypes = Array.isArray(card?.sub_type) ? card.sub_type : [];
  const isLegendary = superTypes.includes("Legendary");
  const isCreature = types.includes("Creature");
  const isVehicle = subTypes.some((subtype) => subtype.toLowerCase() === "vehicle");
  return isLegendary && (isCreature || isVehicle);
}

function collapseCardCopies(cards) {
  const grouped = new Map();

  for (const card of cards) {
    const key = JSON.stringify([
      card.name,
      card.mana_cost || "",
      Array.isArray(card.card_type) ? card.card_type : [],
      Array.isArray(card.super_type) ? card.super_type : [],
      Array.isArray(card.sub_type) ? card.sub_type : [],
      card.oracle_text || "",
    ]);
    const entry = grouped.get(key);
    if (entry) {
      entry.quantity += 1;
      continue;
    }
    grouped.set(key, { card, quantity: 1 });
  }

  return [...grouped.values()].sort((a, b) => a.card.name.localeCompare(b.card.name));
}

const commanderSection = computed(() => collapseCardCopies(commanderCards(deck.value?.commander)));

const mainDeckSections = computed(() => {
  const cards = Array.isArray(deck.value?.cards) ? deck.value.cards : [];
  const groupedByType = new Map();

  for (const card of cards) {
    const type = primaryType(card);
    const bucket = groupedByType.get(type) || [];
    bucket.push(card);
    groupedByType.set(type, bucket);
  }

  return typeDisplayOrder
    .map((type) => ({
      title: type,
      entries: collapseCardCopies(groupedByType.get(type) || []),
    }))
    .filter((section) => section.entries.length > 0);
});

const deckCardTotal = computed(() => Array.isArray(deck.value?.cards) ? deck.value.cards.length : 0);
const manaCurve = computed(() => {
  const cards = Array.isArray(deck.value?.cards) ? deck.value.cards : [];
  if (cards.length === 0) {
    return 0;
  }

  const total = cards.reduce((sum, card) => sum + (Number(card.mana_value) || 0), 0);
  return (total / cards.length).toFixed(2);
});

const pipCounts = computed(() => {
  const cards = Array.isArray(deck.value?.cards) ? deck.value.cards : [];
  const totals = { W: 0, U: 0, B: 0, R: 0, G: 0 };

  for (const card of cards) {
    const manaCost = typeof card.mana_cost === "string" ? card.mana_cost : "";
    for (const pip of ["W", "U", "B", "R", "G"]) {
      const matches = manaCost.match(new RegExp(`\\{${pip}\\}`, "g"));
      totals[pip] += matches ? matches.length : 0;
    }
  }

  return totals;
});

async function handleAddCard() {
  if (isAddingCard.value || isUpdatingDeck.value) {
    return;
  }

  const trimmed = searchName.value.trim();
  if (!trimmed) {
    showError("Enter a card name first.");
    return;
  }

  try {
    isAddingCard.value = true;
    deck.value = await addCardToDeckCommand(normalizeDeckId(props.deckId), trimmed);
    searchName.value = "";
    showSuccess(`Added "${trimmed}" to ${deck.value.name}`);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isAddingCard.value = false;
  }
}

async function handleAddCopy(cardName) {
  try {
    isUpdatingDeck.value = true;
    deck.value = await addCardToDeckCommand(normalizeDeckId(props.deckId), cardName);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleRemoveCopy(cardId, cardName) {
  try {
    isUpdatingDeck.value = true;
    deck.value = await removeCardFromDeckCommand(normalizeDeckId(props.deckId), cardId);
    showSuccess(`Removed one "${cardName}"`);
  } catch (e) {
    showError(`Failed to remove card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleSetCommander(cardId, cardName) {
  try {
    isUpdatingDeck.value = true;
    deck.value = await setDeckCommanderCommand(normalizeDeckId(props.deckId), cardId);
    showSuccess(`Set "${cardName}" as commander`);
  } catch (e) {
    showError(`Failed to set commander: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleRemoveCommander(cardName) {
  try {
    isUpdatingDeck.value = true;
    deck.value = await removeDeckCommanderCommand(normalizeDeckId(props.deckId));
    showSuccess(`Removed "${cardName}" as commander`);
  } catch (e) {
    showError(`Failed to remove commander: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleDeleteCommander(cardName) {
  try {
    isUpdatingDeck.value = true;
    deck.value = await deleteDeckCommanderCommand(normalizeDeckId(props.deckId));
    showSuccess(`Removed "${cardName}" from the deck`);
  } catch (e) {
    showError(`Failed to remove commander card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

watch(() => props.deckId, loadDeck);
onMounted(loadDeck);
</script>

<template>
  <v-container class="deck-editor-page">
    <section class="deck-editor-hero">
      <div class="hero-content">
        <h1>{{ deck?.name || "Deck" }}</h1>
        <div class="hero-actions">
          <v-text-field
            v-model="searchName"
            class="deck-search"
            label="Search and add a card"
            density="comfortable"
            hide-details
            @keyup.enter="handleAddCard"
          />
          <v-btn
            class="add-card-btn"
            :prepend-icon="mdiPlus"
            :loading="isAddingCard"
            @click="handleAddCard"
          >
            Add Card
          </v-btn>
          <v-btn
            class="refresh-btn"
            variant="outlined"
            :prepend-icon="mdiRefresh"
            :loading="isLoading"
            @click="loadDeck"
          >
            Refresh
          </v-btn>
        </div>
      </div>
    </section>

    <div v-if="loadError" class="feedback feedback--error">
      <v-icon :icon="mdiAlertCircleOutline" size="18"></v-icon>
      <span>{{ loadError }}</span>
    </div>

    <template v-else-if="deck">
      <section class="deck-metrics">
        <article class="metric-card">
          <span class="metric-label">Cards</span>
          <strong>{{ deckCardTotal }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Average MV</span>
          <strong>{{ manaCurve }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">White Pips</span>
          <strong>{{ pipCounts.W }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Blue Pips</span>
          <strong>{{ pipCounts.U }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Black Pips</span>
          <strong>{{ pipCounts.B }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Red Pips</span>
          <strong>{{ pipCounts.R }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Green Pips</span>
          <strong>{{ pipCounts.G }}</strong>
        </article>
      </section>

      <section class="deck-layout">
        <aside class="deck-sidebar">
          <section class="deck-panel">
            <div class="panel-heading">
              <v-icon :icon="mdiCrownOutline" size="18"></v-icon>
              <h2>Command Zone</h2>
            </div>
            <div v-if="commanderSection.length > 0" class="deck-list">
              <DeckCardRow
                v-for="entry in commanderSection"
                :key="`commander-${entry.card.id}`"
                :card="entry.card"
                :quantity="entry.quantity"
                :editable="true"
                :can-remove-commander="true"
                @add-copy="handleAddCopy(entry.card.name)"
                @remove-copy="handleDeleteCommander(entry.card.name)"
                @remove-commander="handleRemoveCommander(entry.card.name)"
              />
            </div>
            <p v-else class="empty-copy">No commander selected yet.</p>
          </section>
        </aside>

        <main class="deck-main">
          <section class="deck-panel">
            <div class="panel-heading">
              <v-icon :icon="mdiCardsOutline" size="18"></v-icon>
              <h2>Mainboard</h2>
            </div>

            <div v-if="mainDeckSections.length > 0" class="deck-sections">
              <section
                v-for="section in mainDeckSections"
                :key="section.title"
                class="deck-type-section"
              >
                <header class="deck-type-section__header">
                  <h3>{{ section.title }}</h3>
                  <span>{{ section.entries.reduce((sum, entry) => sum + entry.quantity, 0) }} cards</span>
                </header>
                <div class="deck-list">
                  <DeckCardRow
                    v-for="entry in section.entries"
                    :key="`${section.title}-${entry.card.id}`"
                    :card="entry.card"
                    :quantity="entry.quantity"
                    :can-set-commander="canSetCommander(entry.card)"
                    @add-copy="handleAddCopy(entry.card.name)"
                    @remove-copy="handleRemoveCopy(entry.card.id, entry.card.name)"
                    @set-commander="handleSetCommander(entry.card.id, entry.card.name)"
                  />
                </div>
              </section>
            </div>

            <div v-else class="empty-state">
              <h3>This deck is empty</h3>
              <p>Add cards to the deck and they will appear here in grouped sections.</p>
            </div>
          </section>
        </main>
      </section>
    </template>
    <v-snackbar
      v-model="snackbarVisible"
      :color="snackbarColor"
      :timeout="2500"
    >
      {{ snackbarMessage }}
    </v-snackbar>
  </v-container>
</template>

<style scoped>
.deck-editor-page {
  max-width: 1440px;
  padding: 28px;
  color: #132032;
}

.deck-editor-hero {
  padding: 28px 32px;
  margin-bottom: 20px;
  border-radius: 28px;
  background:
    radial-gradient(circle at top left, rgba(187, 214, 255, 0.9), transparent 34%),
    linear-gradient(135deg, #fbfcff 0%, #eef3fb 48%, #e5ecf7 100%);
  border: 1px solid rgba(34, 53, 84, 0.08);
}

.hero-content {
  display: grid;
  gap: 16px;
}

.deck-editor-hero h1 {
  margin: 0;
  font-size: clamp(2rem, 3vw, 3.2rem);
  line-height: 1;
}

.hero-actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.deck-search {
  min-width: 320px;
  max-width: 520px;
}

.refresh-btn {
  flex: 0 0 auto;
}

.add-card-btn {
  flex: 0 0 auto;
}

.feedback {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  margin-bottom: 20px;
  border-radius: 16px;
}

.feedback--error {
  background: #feeceb;
  color: #8b2d27;
}

.deck-metrics {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 14px;
  margin-bottom: 20px;
}

.metric-card {
  padding: 18px 20px;
  border-radius: 20px;
  background: #fff;
  border: 1px solid rgba(27, 42, 63, 0.08);
  box-shadow: 0 16px 30px rgba(24, 37, 58, 0.05);
}

.metric-label {
  display: block;
  margin-bottom: 10px;
  font-size: 0.82rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #687892;
}

.metric-card strong {
  font-size: 1.8rem;
}

.deck-layout {
  display: grid;
  grid-template-columns: minmax(280px, 340px) minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}

.deck-sidebar,
.deck-main,
.deck-sections {
  display: grid;
  gap: 20px;
}

.deck-panel {
  padding: 20px;
  border-radius: 24px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.96) 0%, rgba(247, 250, 255, 0.98) 100%);
  border: 1px solid rgba(27, 42, 63, 0.08);
  box-shadow: 0 20px 40px rgba(20, 31, 48, 0.05);
}

.panel-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.panel-heading h2,
.deck-type-section__header h3 {
  margin: 0;
}

.deck-list {
  display: grid;
  gap: 10px;
}

.deck-type-section {
  display: grid;
  gap: 12px;
}

.deck-type-section__header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: baseline;
}

.deck-type-section__header span {
  color: #607089;
  font-size: 0.9rem;
}

.empty-copy,
.empty-state p {
  margin: 0;
  color: #5f6f86;
  line-height: 1.5;
}

.empty-state {
  padding: 36px 12px 18px;
  text-align: center;
}

.empty-state h3 {
  margin: 0 0 8px;
}

@media (max-width: 1080px) {
  .deck-metrics {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .deck-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 720px) {
  .deck-editor-page {
    padding: 16px;
  }

  .deck-editor-hero {
    padding: 22px;
  }

  .hero-actions {
    width: 100%;
    justify-content: stretch;
  }

  .deck-search {
    width: 100%;
    min-width: 0;
  }

  .deck-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
