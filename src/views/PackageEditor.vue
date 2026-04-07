<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";
import {
  mdiAlertCircleOutline,
  mdiCardsOutline,
  mdiPlus,
  mdiCancel,
  mdiGaugeFull,
  mdiChevronLeft,
  mdiDownload,
  mdiExport,
  mdiContentPaste,
  mdiCheck,
} from "@mdi/js";
import {
  addCardToPackageCommand,
  bulkAddCardsToPackageCommand,
  getPackageCommand,
  removeCardFromPackageCommand,
  searchCardSuggestionsCommand,
} from "../api/deckCommands.js";
import DeckCardRow from "../components/DeckCardRow.vue";
import ManaText from "../components/ManaText.vue";

const props = defineProps({
  packageId: {
    type: [String, Number],
    required: true,
  },
});

const router = useRouter();

const packageEntry = ref(null);
const isLoading = ref(false);
const isUpdatingPackage = ref(false);
const loadError = ref("");
const searchName = ref("");
const cardSuggestions = ref([]);
const isSearchingCards = ref(false);
const isSearchFocused = ref(false);
const activeSuggestionIndex = ref(-1);
const isAddingCard = ref(false);
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");
const importDialogVisible = ref(false);
const importText = ref("");
const isImporting = ref(false);
const isPasting = ref(false);
const importErrors = ref([]);
let suggestionSearchTimeout = null;
let searchBlurTimeout = null;
let unlistenImages = null;

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

function normalizePackageId(value) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

function goBack() {
  router.back();
}

async function loadPackage() {
  const normalizedPackageId = normalizePackageId(props.packageId);
  if (normalizedPackageId === null) {
    packageEntry.value = null;
    loadError.value = "Invalid package id.";
    return;
  }

  isLoading.value = true;
  loadError.value = "";

  try {
    packageEntry.value = await getPackageCommand(normalizedPackageId);
  } catch (e) {
    packageEntry.value = null;
    loadError.value = `Failed to load package: ${String(e)}`;
    console.error(e);
  } finally {
    isLoading.value = false;
  }
}

function openImportDialog() {
  importDialogVisible.value = true;
  importText.value = "";
  importErrors.value = [];
}

function closeImportDialog() {
  importDialogVisible.value = false;
  importText.value = "";
  importErrors.value = [];
  isImporting.value = false;
  isPasting.value = false;
}

async function pasteFromClipboard() {
  try {
    isPasting.value = true;
    const text = await navigator.clipboard.readText();
    importText.value = text || "";
    importErrors.value = validateImportLines(importText.value);
  } catch (e) {
    showError("Failed to read clipboard.");
    console.error(e);
  } finally {
    isPasting.value = false;
  }
}

function validateImportLines(text) {
  const lines = text.split(/\r?\n/);
  const errors = [];
  for (let i = 0; i < lines.length; i++) {
    const raw = lines[i];
    const line = raw.trim();
    if (!line) continue;
    const m = line.match(/^(\d+)\s+(.+)$/);
    if (!m) {
      errors.push({ line: i + 1, text: raw });
    }
  }
  return errors;
}

async function handleImport() {
  importErrors.value = validateImportLines(importText.value);
  if (importErrors.value.length > 0) {
    showError("Fix import format errors before importing.");
    return;
  }

  const lines = importText.value
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0);
  if (lines.length === 0) {
    showError("No cards to import.");
    return;
  }

  const normalizedPackageId = normalizePackageId(props.packageId);
  if (normalizedPackageId === null) {
    showError("Invalid package id.");
    return;
  }

  try {
    isImporting.value = true;

    const cardData = [];
    for (const raw of lines) {
      const m = raw.match(/^(\d+)\s+(.+)$/);
      if (!m) continue;
      const qty = Number(m[1]);
      const name = m[2].trim();
      cardData.push([qty, name]);
    }

    if (cardData.length > 0) {
      packageEntry.value = await bulkAddCardsToPackageCommand(normalizedPackageId, cardData);
    }

    showSuccess(`Imported ${lines.length} lines to ${packageEntry.value?.name || "package"}`);
    closeImportDialog();
  } catch (e) {
    console.error(e);
    showError(`Failed to import package: ${String(e)}`);
  } finally {
    isImporting.value = false;
  }
}

async function exportPackageAsText() {
  if (!packageEntry.value) return;

  try {
    const lines = [];
    const entries = collapseCardCopies(packageEntry.value.cards || []);
    for (const entry of entries) {
      lines.push(`${entry.quantity} ${entry.card.name}`);
    }

    const text = lines.join("\n");
    await navigator.clipboard.writeText(text);
    showSuccess("Package exported to clipboard.");
  } catch (e) {
    showError("Failed to export package.");
    console.error(e);
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

function clearSuggestions() {
  cardSuggestions.value = [];
  activeSuggestionIndex.value = -1;
}

function hideSuggestions() {
  isSearchFocused.value = false;
  clearSuggestions();
}

function handleSearchFocus() {
  if (searchBlurTimeout) {
    clearTimeout(searchBlurTimeout);
    searchBlurTimeout = null;
  }
  isSearchFocused.value = true;
}

function handleSearchBlur() {
  searchBlurTimeout = setTimeout(() => {
    hideSuggestions();
  }, 120);
}

function selectSuggestion(suggestion) {
  searchName.value = suggestion.name;
  isSearchFocused.value = false;
  clearSuggestions();
}

function moveSuggestion(direction) {
  if (cardSuggestions.value.length === 0) {
    return;
  }

  const nextIndex = activeSuggestionIndex.value + direction;
  if (nextIndex < 0) {
    activeSuggestionIndex.value = cardSuggestions.value.length - 1;
    return;
  }
  if (nextIndex >= cardSuggestions.value.length) {
    activeSuggestionIndex.value = 0;
    return;
  }
  activeSuggestionIndex.value = nextIndex;
}

async function handleSearchEnter() {
  if (cardSuggestions.value.length > 0 && activeSuggestionIndex.value >= 0) {
    selectSuggestion(cardSuggestions.value[activeSuggestionIndex.value]);
  }

  await handleAddCard();
}

function primaryType(card) {
  const types = Array.isArray(card.card_type) ? card.card_type : [];
  return typeDisplayOrder.find((type) => types.includes(type)) || "Other";
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
      entry.cardIds.push(card.id);
      continue;
    }
    grouped.set(key, { key, card, quantity: 1, cardIds: [card.id] });
  }

  return [...grouped.values()].sort((a, b) => a.card.name.localeCompare(b.card.name));
}

const packageCards = computed(() => (Array.isArray(packageEntry.value?.cards) ? packageEntry.value.cards : []));
const packageSections = computed(() => {
  const groupedByType = new Map();

  for (const card of packageCards.value) {
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
const packageCardTotal = computed(() => packageCards.value.length);
const manaCurve = computed(() => {
  if (packageCards.value.length === 0) {
    return "0.00";
  }

  const total = packageCards.value.reduce((sum, card) => sum + (Number(card.mana_value) || 0), 0);
  return (total / packageCards.value.length).toFixed(2);
});
const pipCounts = computed(() => {
  const totals = { W: 0, U: 0, B: 0, R: 0, G: 0 };

  for (const card of packageCards.value) {
    const manaCost = typeof card.mana_cost === "string" ? card.mana_cost : "";
    for (const pip of ["W", "U", "B", "R", "G"]) {
      const matches = manaCost.match(new RegExp(`\\{${pip}\\}`, "g"));
      totals[pip] += matches ? matches.length : 0;
    }
  }

  return totals;
});
const showCardSuggestions = computed(() => (
  isSearchFocused.value &&
  cardSuggestions.value.length > 0 &&
  searchName.value.trim().length >= 2
));

async function handleAddCard() {
  if (isAddingCard.value || isUpdatingPackage.value) {
    return;
  }

  const normalizedPackageId = normalizePackageId(props.packageId);
  if (normalizedPackageId === null) {
    showError("Invalid package id.");
    return;
  }

  const trimmed = searchName.value.trim();
  if (!trimmed) {
    showError("Enter a card name first.");
    return;
  }

  try {
    isAddingCard.value = true;
    packageEntry.value = await addCardToPackageCommand(normalizedPackageId, trimmed);
    searchName.value = "";
    clearSuggestions();
    showSuccess(`Added "${trimmed}" to ${packageEntry.value.name}`);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isAddingCard.value = false;
  }
}

async function handleAddCopy(cardName) {
  const normalizedPackageId = normalizePackageId(props.packageId);
  if (normalizedPackageId === null) {
    showError("Invalid package id.");
    return;
  }

  try {
    isUpdatingPackage.value = true;
    packageEntry.value = await addCardToPackageCommand(normalizedPackageId, cardName);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingPackage.value = false;
  }
}

async function handleRemoveCopy(cardId, cardName) {
  const normalizedPackageId = normalizePackageId(props.packageId);
  if (normalizedPackageId === null) {
    showError("Invalid package id.");
    return;
  }

  try {
    isUpdatingPackage.value = true;
    packageEntry.value = await removeCardFromPackageCommand(normalizedPackageId, cardId);
    showSuccess(`Removed one "${cardName}"`);
  } catch (e) {
    showError(`Failed to remove card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingPackage.value = false;
  }
}

watch(() => props.packageId, loadPackage);
watch(searchName, (value) => {
  if (suggestionSearchTimeout) {
    clearTimeout(suggestionSearchTimeout);
  }

  const trimmed = value.trim();
  if (trimmed.length < 2) {
    isSearchingCards.value = false;
    clearSuggestions();
    return;
  }

  suggestionSearchTimeout = setTimeout(async () => {
    try {
      isSearchingCards.value = true;
      const suggestions = await searchCardSuggestionsCommand(trimmed);
      cardSuggestions.value = suggestions;
      activeSuggestionIndex.value = suggestions.length > 0 ? 0 : -1;
    } catch (e) {
      console.error(e);
      clearSuggestions();
    } finally {
      isSearchingCards.value = false;
    }
  }, 180);
});

onMounted(async () => {
  await loadPackage();
  unlistenImages = await listen("images-updated", () => {
    console.log("Images updated event received in Package Editor, reloading...");
    loadPackage();
  });
});

onUnmounted(() => {
  if (unlistenImages) {
    unlistenImages();
  }
});
</script>

<template>
  <v-container class="deck-editor-page">
    <section class="deck-editor-hero">
      <div class="hero-content">
        <div class="d-flex align-center gap-4">
          <v-btn icon variant="text" @click="goBack" color="primary">
            <v-icon :icon="mdiChevronLeft" size="32"></v-icon>
          </v-btn>
          <h1 class="text-primary">{{ packageEntry?.name || "Package" }}</h1>
        </div>
        <div class="hero-actions">
          <div class="deck-search-wrap">
            <v-text-field
              v-model="searchName"
              class="deck-search"
              label="Search and add a card"
              density="comfortable"
              variant="outlined"
              hide-details
              :loading="isSearchingCards"
              @focus="handleSearchFocus"
              @blur="handleSearchBlur"
              @keydown.enter.prevent="handleSearchEnter"
              @keydown.down.prevent="moveSuggestion(1)"
              @keydown.up.prevent="moveSuggestion(-1)"
              @keydown.esc="hideSuggestions"
            />

            <div v-if="showCardSuggestions" class="deck-search-suggestions">
              <button
                v-for="(suggestion, index) in cardSuggestions"
                :key="`${suggestion.name}-${index}`"
                class="deck-search-suggestion"
                :class="{
                  'deck-search-suggestion--active': index === activeSuggestionIndex,
                  'deck-search-suggestion--illegal':
                    suggestion.commander_legality && suggestion.commander_legality !== 'legal',
                  'deck-search-suggestion--game-changer':
                    suggestion.game_changer &&
                    !(suggestion.commander_legality && suggestion.commander_legality !== 'legal'),
                }"
                type="button"
                @mousedown.prevent="selectSuggestion(suggestion)"
              >
                <div class="deck-search-suggestion__top">
                  <div class="d-flex align-center flex-grow-1">
                    <span>{{ suggestion.name }}</span>
                    <span
                      v-if="suggestion.game_changer"
                      class="suggestion-pill suggestion-pill--game-changer ml-1"
                    >
                      GAME CHANGER
                    </span>
                    <span
                      v-if="suggestion.commander_legality && suggestion.commander_legality !== 'legal'"
                      class="suggestion-pill suggestion-pill--illegal ml-1"
                    >
                      {{ suggestion.commander_legality.toUpperCase().replace("_", " ") }}
                    </span>
                    <v-icon
                      v-if="suggestion.game_changer"
                      :icon="mdiGaugeFull"
                      size="14"
                      color="amber-darken-2"
                      title="Game Changer"
                      class="ml-1"
                    ></v-icon>
                    <v-icon
                      v-if="suggestion.commander_legality && suggestion.commander_legality !== 'legal'"
                      :icon="mdiCancel"
                      size="14"
                      color="error"
                      title="Banned"
                      class="ml-1"
                    ></v-icon>
                  </div>
                  <ManaText v-if="suggestion.mana_cost" :text="suggestion.mana_cost" :cost="true" />
                </div>
                <span class="deck-search-suggestion__type">{{ suggestion.type_line }}</span>
              </button>
            </div>
          </div>

          <v-btn class="add-card-btn" :prepend-icon="mdiPlus" :loading="isAddingCard" @click="handleAddCard">
            Add Card
          </v-btn>
          <v-btn
            class="import-btn"
            variant="outlined"
            :prepend-icon="mdiDownload"
            @click="openImportDialog"
          >
            Import Decklist
          </v-btn>
          <v-btn
            class="export-btn"
            variant="outlined"
            :prepend-icon="mdiExport"
            :disabled="!packageEntry || packageCardTotal === 0"
            @click="exportPackageAsText"
          >
            Export Package
          </v-btn>
        </div>
      </div>
    </section>

    <div v-if="loadError" class="feedback feedback--error">
      <v-icon :icon="mdiAlertCircleOutline" size="18"></v-icon>
      <span>{{ loadError }}</span>
    </div>

    <template v-else-if="packageEntry">
      <section class="deck-metrics">
        <article class="metric-card">
          <span class="metric-label">Cards</span>
          <strong>{{ packageCardTotal }}</strong>
        </article>
        <article class="metric-card">
          <span class="metric-label">Average MV</span>
          <strong>{{ manaCurve }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{W}" :cost="true" />
          <strong>{{ pipCounts.W }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{U}" :cost="true" />
          <strong>{{ pipCounts.U }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{B}" :cost="true" />
          <strong>{{ pipCounts.B }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{R}" :cost="true" />
          <strong>{{ pipCounts.R }}</strong>
        </article>
        <article class="metric-card">
          <ManaText class="metric-label metric-label--symbol" text="{G}" :cost="true" />
          <strong>{{ pipCounts.G }}</strong>
        </article>
      </section>

      <section class="deck-layout deck-layout--single">
        <main class="deck-main">
          <section class="deck-panel">
            <div class="panel-heading">
              <v-icon :icon="mdiCardsOutline" size="18"></v-icon>
              <h2>Package Cards</h2>
            </div>

            <div v-if="packageSections.length > 0" class="deck-sections">
              <section v-for="section in packageSections" :key="section.title" class="deck-type-section">
                <header class="deck-type-section__header">
                  <h3>{{ section.title }}</h3>
                  <span>{{ section.entries.reduce((sum, entry) => sum + entry.quantity, 0) }} cards</span>
                </header>
                <div class="deck-list">
                  <DeckCardRow
                    v-for="entry in section.entries"
                    :key="`${section.title}-${entry.key}`"
                    :card="entry.card"
                    :quantity="entry.quantity"
                    @add-copy="handleAddCopy(entry.card.name)"
                    @remove-copy="handleRemoveCopy(entry.cardIds[entry.cardIds.length - 1], entry.card.name)"
                  />
                </div>
              </section>
            </div>

            <div v-else class="empty-state">
              <h3>This package is empty</h3>
              <p>Add cards to the package and they will appear here in grouped sections.</p>
            </div>
          </section>
        </main>
      </section>
    </template>

    <v-snackbar v-model="snackbarVisible" :color="snackbarColor" :timeout="2500">
      {{ snackbarMessage }}
    </v-snackbar>

    <!-- Import Dialog -->
    <v-dialog v-model="importDialogVisible" max-width="600" persistent>
      <v-card rounded="xl" class="pa-4">
        <v-card-title class="d-flex align-center justify-space-between pb-0">
          <span class="text-h5 font-weight-bold">Import Card List</span>
          <v-btn icon variant="text" @click="closeImportDialog" :disabled="isImporting">
            <v-icon :icon="mdiCancel"></v-icon>
          </v-btn>
        </v-card-title>

        <v-card-text class="pt-4">
          <p class="text-body-2 mb-4 opacity-70">
            Paste your card list below. Format: "Quantity CardName" (e.g., "1 Sol Ring").
          </p>

          <v-textarea
            v-model="importText"
            label="Card List"
            placeholder="1 Sol Ring&#10;1 Arcane Signet"
            variant="outlined"
            rows="10"
            auto-grow
            class="import-textarea"
            :error-messages="
              importErrors.length > 0 ? `Format errors on ${importErrors.length} lines` : ''
            "
            persistent-hint
            hint="One card per line: [quantity] [name]"
          ></v-textarea>

          <div v-if="importErrors.length > 0" class="mt-4">
            <div class="text-caption text-error font-weight-bold mb-1">Format Errors:</div>
            <v-list density="compact" class="bg-error-lighten-5 rounded-lg">
              <v-list-item v-for="(err, i) in importErrors.slice(0, 5)" :key="i">
                <template #prepend>
                  <v-icon :icon="mdiAlertCircleOutline" color="error" size="14"></v-icon>
                </template>
                <div class="text-caption text-error">Line {{ err.line }}: "{{ err.text }}"</div>
              </v-list-item>
              <v-list-item v-if="importErrors.length > 5">
                <div class="text-caption text-error">...and {{ importErrors.length - 5 }} more</div>
              </v-list-item>
            </v-list>
          </div>
        </v-card-text>

        <v-card-actions class="pa-4 pt-0">
          <v-btn
            variant="tonal"
            :prepend-icon="mdiContentPaste"
            @click="pasteFromClipboard"
            :loading="isPasting"
            :disabled="isImporting"
          >
            Paste
          </v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="primary"
            variant="flat"
            class="px-6"
            :prepend-icon="mdiCheck"
            @click="handleImport"
            :loading="isImporting"
            :disabled="importErrors.length > 0 || !importText.trim()"
          >
            Import
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<style scoped>
.deck-editor-page {
  max-width: 1440px;
  padding: 28px;
}

.deck-editor-hero {
  padding: 36px 48px;
  background:
    linear-gradient(135deg, rgba(var(--v-theme-primary), 0.08) 0%, rgba(var(--v-theme-surface), 0.95) 100%);
  border-radius: 32px;
  margin-bottom: 24px;
  border: 1px solid rgba(var(--v-theme-primary), 0.15);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.3);
}

.hero-content {
  display: grid;
  gap: 16px;
}

.gap-4 {
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
  min-width: 0;
}

.import-btn,
.export-btn {
  flex: 0 0 auto;
}

.import-textarea {
  max-height: 300px;
  overflow-y: auto !important;
}

.deck-search-wrap {
  position: relative;
  min-width: 320px;
  max-width: 520px;
  width: 100%;
}

.deck-search-suggestions {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 20;
  display: grid;
  gap: 6px;
  padding: 10px;
  border-radius: 18px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 22px 40px rgba(0, 0, 0, 0.4);
}

.deck-search-suggestion {
  display: grid;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.05);
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s ease;
}

.deck-search-suggestion--illegal {
  background: rgba(var(--v-theme-error), 0.1);
  border-color: rgba(var(--v-theme-error), 0.3);
}

.deck-search-suggestion--game-changer {
  background: rgba(255, 193, 7, 0.1);
  border-color: rgba(255, 193, 7, 0.3);
}

.deck-search-suggestion--active,
.deck-search-suggestion:hover {
  background: rgba(var(--v-theme-primary), 0.1);
  border-color: rgba(var(--v-theme-primary), 0.3);
}

.deck-search-suggestion--illegal.deck-search-suggestion--active,
.deck-search-suggestion--illegal:hover {
  background: rgba(var(--v-theme-error), 0.2);
  border-color: rgba(var(--v-theme-error), 0.5);
}

.deck-search-suggestion--game-changer.deck-search-suggestion--active,
.deck-search-suggestion--game-changer:hover {
  background: rgba(255, 193, 7, 0.2);
  border-color: rgba(255, 193, 7, 0.5);
}

.deck-search-suggestion__top {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  font-weight: 700;
}

.deck-search-suggestion__type {
  opacity: 0.7;
  font-size: 0.85rem;
}

.suggestion-pill {
  font-size: 0.6rem;
  font-weight: 800;
  padding: 1px 4px;
  border-radius: 4px;
  letter-spacing: 0.04em;
  flex: 0 0 auto;
}

.suggestion-pill--illegal {
  background: #991b1b;
  color: #fff;
}

.suggestion-pill--game-changer {
  background: #c2410c;
  color: #fff;
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
  background: rgba(var(--v-theme-error), 0.1);
  color: rgb(var(--v-theme-error));
  border: 1px solid rgba(var(--v-theme-error), 0.2);
}

.deck-metrics {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 14px;
  margin-bottom: 20px;
}

.metric-card {
  padding: 16px 18px;
  border-radius: 20px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 16px 30px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.metric-label {
  display: block;
  font-size: 0.82rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.7;
}

.metric-card strong {
  font-size: 1.6rem;
  width: 100%;
}

.metric-label--symbol {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-height: 2.3rem;
}

.metric-label--symbol :deep(.ms-cost) {
  width: 2.3rem;
  height: 2.3rem;
  line-height: 2.3rem;
  font-size: 1.4rem;
}

.deck-layout {
  display: grid;
  gap: 20px;
  align-items: start;
}

.deck-layout--single {
  grid-template-columns: minmax(0, 1fr);
}

.deck-main,
.deck-sections {
  display: grid;
  gap: 20px;
}

.deck-panel {
  padding: 20px;
  border-radius: 24px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
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
  opacity: 0.7;
  font-size: 0.9rem;
}

.empty-state {
  padding: 36px 12px 18px;
  text-align: center;
}

.empty-state h3 {
  margin: 0 0 8px;
}

.empty-state p {
  margin: 0;
  opacity: 0.7;
  line-height: 1.5;
}

@media (max-width: 1080px) {
  .deck-metrics {
    grid-template-columns: repeat(3, minmax(0, 1fr));
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
  }

  .deck-search-wrap {
    min-width: 0;
  }

  .deck-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
