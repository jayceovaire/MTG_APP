<script setup>
import { computed, onMounted, ref, watch } from "vue";
import {
  mdiAlertCircleOutline,
  mdiCardsOutline,
  mdiCrownOutline,
  mdiPackageVariantClosedPlus,
  mdiPlus,
  mdiDownload,
  mdiGaugeFull,
  mdiCancel,
} from "@mdi/js";
import {
  addCardToDeckCommand,
  bulkAddCardsToDeckCommand,
  addCardToPackageCommand,
  addPackageToDeckCommand,
  createPackageCommand,
  deleteDeckCommanderCommand,
  getDeckCommand,
  getPackagesCommand,
  removeDeckCommanderCommand,
  removeCardFromDeckCommand,
  searchCardSuggestionsCommand,
  setDeckCommanderCommand,
  setDeckPartnerCommand,
  removeDeckPartnerCommand,
} from "../api/deckCommands.js";
import DeckCardRow from "../components/DeckCardRow.vue";
import ManaText from "../components/ManaText.vue";

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
const cardSuggestions = ref([]);
const isSearchingCards = ref(false);
const isSearchFocused = ref(false);
const activeSuggestionIndex = ref(-1);
const isAddingCard = ref(false);
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");
const packages = ref([]);
const isLoadingPackages = ref(false);
const packageDialogVisible = ref(false);
const packageDialogMode = ref("card");
const selectedPackageId = ref(null);
const newPackageName = ref("");
const pendingPackageCardName = ref("");
const isSubmittingPackageAction = ref(false);
const isCreatingPackage = ref(false);
// Import deck dialog state
const importDialogVisible = ref(false);
const importText = ref("");
const importErrors = ref([]);
const isImporting = ref(false);
const isPasting = ref(false);
let suggestionSearchTimeout = null;
let searchBlurTimeout = null;

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
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

async function loadDeck() {
  const normalizedDeckId = normalizeDeckId(props.deckId);

  if (normalizedDeckId === null) {
    deck.value = null;
    loadError.value = "Invalid deck id.";
    return;
  }

  isLoading.value = true;
  loadError.value = "";

  try {
    deck.value = await getDeckCommand(normalizedDeckId);
  } catch (e) {
    deck.value = null;
    loadError.value = `Failed to load deck: ${String(e)}`;
    console.error(e);
  } finally {
    isLoading.value = false;
  }
}

async function loadPackages() {
  isLoadingPackages.value = true;

  try {
    packages.value = await getPackagesCommand();
  } catch (e) {
    showError(`Failed to load packages: ${String(e)}`);
    console.error(e);
  } finally {
    isLoadingPackages.value = false;
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

function closePackageDialog() {
  packageDialogVisible.value = false;
  selectedPackageId.value = null;
  newPackageName.value = "";
  pendingPackageCardName.value = "";
  isSubmittingPackageAction.value = false;
  isCreatingPackage.value = false;
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

  const lines = importText.value.split(/\r?\n/).map((l) => l.trim()).filter((l) => l.length > 0);
  if (lines.length === 0) {
    showError("No cards to import.");
    return;
  }

  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
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
      deck.value = await bulkAddCardsToDeckCommand(normalizedDeckId, cardData);
    }

    showSuccess(`Imported ${lines.length} lines to ${deck.value?.name || 'deck'}`);
    closeImportDialog();
  } catch (e) {
    console.error(e);
    showError(`Failed to import deck: ${String(e)}`);
  } finally {
    isImporting.value = false;
  }
}

function openAddCardToPackageDialog(cardName) {
  packageDialogMode.value = "card";
  pendingPackageCardName.value = cardName;
  selectedPackageId.value = packages.value[0]?.id ?? null;
  newPackageName.value = "";
  packageDialogVisible.value = true;
}

function openAddPackageToDeckDialog() {
  packageDialogMode.value = "deck";
  pendingPackageCardName.value = "";
  selectedPackageId.value = packages.value[0]?.id ?? null;
  newPackageName.value = "";
  packageDialogVisible.value = true;
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

/** Stable id match (avoids undefined===undefined treating every card as the commander). */
function sameCardId(a, b) {
  if (a == null || b == null) {
    return false;
  }
  const na = Number(a);
  const nb = Number(b);
  if (!Number.isFinite(na) || !Number.isFinite(nb)) {
    return false;
  }
  return na === nb;
}

function commanderCards(commander) {
  if (!commander) {
    return [];
  }

  if (typeof commander === "string") {
    return commander === "None" ? [] : [];
  }

  // Serde externally-tagged None: { "None": null }
  if (typeof commander === "object" && commander !== null && Object.prototype.hasOwnProperty.call(commander, "None")) {
    return [];
  }

  const single = commander.Single ?? commander.single;
  if (single) {
    return [single];
  }

  const partner = commander.Partner ?? commander.partner;
  if (Array.isArray(partner)) {
    return partner;
  }

  return [];
}

function primaryType(card) {
  const types = Array.isArray(card.card_type) ? card.card_type : [];
  return typeDisplayOrder.find((type) => types.includes(type)) || "Other";
}

function isCommanderCard(card) {
  if (card == null || card.id == null) {
    return false;
  }
  return commanderCards(deck.value?.commander).some((c) => c != null && sameCardId(c.id, card.id));
}

/** Rules only (used for current commander in canSetPartner, etc.). */
function canBeCommander(card) {
  const types = Array.isArray(card?.card_type) ? card.card_type : [];
  const superTypes = Array.isArray(card?.super_type) ? card.super_type : [];
  const subTypes = Array.isArray(card?.sub_type) ? card.sub_type : [];
  const oracleText = typeof card?.oracle_text === "string" ? card.oracle_text.toLowerCase() : "";

  const isLegendary = superTypes.includes("Legendary");
  const isCreature = types.includes("Creature");
  const isVehicle = subTypes.some((subtype) => subtype.toLowerCase() === "vehicle");
  const isBackground = subTypes.some((subtype) => subtype.toLowerCase() === "background");

  // Standard case: Legendary Creature or Legendary Vehicle
  if (isLegendary && (isCreature || isVehicle)) {
    return true;
  }

  // Special case: Backgrounds
  if (isLegendary && isBackground) {
    return true;
  }

  // Cards with "can be your commander" text (includes some planeswalkers)
  if (oracleText.includes("can be your commander")) {
    return true;
  }

  return false;
}

/** Whether the row may show "Set as Commander" (mainboard / not already in command zone). */
function canSetCommander(card) {
  if (isCommanderCard(card)) {
    return false;
  }
  return canBeCommander(card);
}

function hasPartnerMechanic(card) {
  const oracleText = typeof card?.oracle_text === "string" ? card.oracle_text : "";
  const subTypes = Array.isArray(card?.sub_type) ? card.sub_type : [];
  const lower = oracleText.toLowerCase();

  // Special case: Backgrounds
  if (subTypes.some((s) => s.toLowerCase() === "background")) {
    return true;
  }

  // Check each line for partner-like mechanics
  const hasPartnerLine = oracleText
    .split("\n")
    .map((line) => line.trim().toLowerCase())
    .some((line) =>
      line === "partner" ||
      line.startsWith("partner with ") ||
      line === "friends forever" ||
      line === "choose a background" ||
      line === "doctor's companion"
    );

  if (hasPartnerLine) {
    return true;
  }

  // Also check for "partner" as a standalone word anywhere in case it's formatted differently
  return lower.includes("partner") ||
         lower.includes("friends forever") ||
         lower.includes("choose a background") ||
         lower.includes("doctor's companion");
}

function canSetPartner(card) {
  if (isCommanderCard(card)) {
    return false;
  }
  if (!canBeCommander(card) || !hasPartnerMechanic(card)) {
    return false;
  }

  const commander = deck.value?.commander;
  if (!commander || commander === "None") {
    return false;
  }

  const single = commander.Single ?? commander.single;
  if (!single) {
    return false;
  }

  return hasPartnerMechanic(single) && canBeCommander(single);
}

/** Only for a card currently shown as partner commander in the command zone (not mainboard). */
function canRemovePartner(card) {
  const commander = deck.value?.commander;
  if (!commander || commander === "None") {
    return false;
  }
  const partner = commander.Partner ?? commander.partner;
  if (!Array.isArray(partner)) {
    return false;
  }
  if (card == null || card.id == null) {
    return false;
  }
  const isThisPartner = partner.some((partnerCard) => sameCardId(partnerCard?.id, card.id));
  if (!isThisPartner) {
    return false;
  }
  return canBeCommander(card) && hasPartnerMechanic(card);
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
const showCardSuggestions = computed(() => (
  isSearchFocused.value &&
  cardSuggestions.value.length > 0 &&
  searchName.value.trim().length >= 2
));
const allDeckCards = computed(() => {
  const mainboardCards = Array.isArray(deck.value?.cards) ? deck.value.cards : [];
  return [...mainboardCards, ...commanderCards(deck.value?.commander)];
});

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

const deckCardTotal = computed(() => allDeckCards.value.length);
const manaCurve = computed(() => {
  const cards = allDeckCards.value;
  if (cards.length === 0) {
    return 0;
  }

  const total = cards.reduce((sum, card) => sum + (Number(card.mana_value) || 0), 0);
  return (total / cards.length).toFixed(2);
});

const pipCounts = computed(() => {
  const cards = allDeckCards.value;
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

  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  const trimmed = searchName.value.trim();
  if (!trimmed) {
    showError("Enter a card name first.");
    return;
  }

  try {
    isAddingCard.value = true;
    deck.value = await addCardToDeckCommand(normalizedDeckId, trimmed);
    searchName.value = "";
    clearSuggestions();
    showSuccess(`Added "${trimmed}" to ${deck.value.name}`);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isAddingCard.value = false;
  }
}

async function handleAddCopy(cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await addCardToDeckCommand(normalizedDeckId, cardName);
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleRemoveCopy(cardId, cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await removeCardFromDeckCommand(normalizedDeckId, cardId);
    showSuccess(`Removed one "${cardName}"`);
  } catch (e) {
    showError(`Failed to remove card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleSetCommander(cardId, cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await setDeckCommanderCommand(normalizedDeckId, cardId);
    showSuccess(`Set "${cardName}" as commander`);
  } catch (e) {
    showError(`Failed to set commander: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleSetPartner(cardId, cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await setDeckPartnerCommand(normalizedDeckId, cardId);
    showSuccess(`Set "${cardName}" as partner commander`);
  } catch (e) {
    showError(`Failed to set partner: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleRemovePartner(cardId, cardName){
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null){
    showError("Invalid deck id.");
    return;
  }
  try {
    isUpdatingDeck.value = true;
    deck.value = await removeDeckPartnerCommand(normalizedDeckId,cardId);
    showSuccess(`Removed "${cardName}" as partner commander`);
  } catch (e){
    showError(`Failed to remove partner: ${String(e)}`);
    console.error(e)
  } finally {
    isUpdatingDeck.value = false;
  }

}


async function handleRemoveCommander(cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await removeDeckCommanderCommand(normalizedDeckId);
    showSuccess(`Removed "${cardName}" as commander`);
  } catch (e) {
    showError(`Failed to remove commander: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleDeleteCommander(cardName) {
  const normalizedDeckId = normalizeDeckId(props.deckId);
  if (normalizedDeckId === null) {
    showError("Invalid deck id.");
    return;
  }

  try {
    isUpdatingDeck.value = true;
    deck.value = await deleteDeckCommanderCommand(normalizedDeckId);
    showSuccess(`Removed "${cardName}" from the deck`);
  } catch (e) {
    showError(`Failed to remove commander card: ${String(e)}`);
    console.error(e);
  } finally {
    isUpdatingDeck.value = false;
  }
}

async function handleCreatePackage() {
  const trimmed = newPackageName.value.trim();
  if (!trimmed) {
    showError("Enter a package name first.");
    return;
  }

  try {
    isCreatingPackage.value = true;
    const newPackage = await createPackageCommand(trimmed);
    packages.value.push(newPackage);
    selectedPackageId.value = newPackage.id;
    newPackageName.value = "";
    showSuccess(`Created package "${newPackage.name}"`);
  } catch (e) {
    showError(`Failed to create package: ${String(e)}`);
    console.error(e);
  } finally {
    isCreatingPackage.value = false;
  }
}

async function handleSubmitPackageDialog() {
  if (isSubmittingPackageAction.value) {
    return;
  }

  const packageId = Number(selectedPackageId.value);
  if (!Number.isInteger(packageId) || packageId <= 0) {
    showError("Select a package first.");
    return;
  }

  try {
    isSubmittingPackageAction.value = true;

    if (packageDialogMode.value === "card") {
      const updatedPackage = await addCardToPackageCommand(packageId, pendingPackageCardName.value);
      const index = packages.value.findIndex((entry) => entry.id === updatedPackage.id);
      if (index >= 0) {
        packages.value[index] = updatedPackage;
      } else {
        packages.value.push(updatedPackage);
      }
      showSuccess(`Added "${pendingPackageCardName.value}" to ${updatedPackage.name}`);
    } else {
      const normalizedDeckId = normalizeDeckId(props.deckId);
      if (normalizedDeckId === null) {
        showError("Invalid deck id.");
        return;
      }
      const selectedPackage = packages.value.find((entry) => entry.id === packageId);
      deck.value = await addPackageToDeckCommand(normalizedDeckId, packageId);
      showSuccess(`Added package "${selectedPackage?.name || "Package"}" to ${deck.value.name}`);
    }

    closePackageDialog();
  } catch (e) {
    showError(`Failed to apply package action: ${String(e)}`);
    console.error(e);
  } finally {
    isSubmittingPackageAction.value = false;
  }
}

function handleAddToPackage(cardName) {
  openAddCardToPackageDialog(cardName);
}

watch(() => props.deckId, loadDeck);
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
  await Promise.all([loadDeck(), loadPackages()]);
});
</script>

<template>
  <v-container class="deck-editor-page">
    <section class="deck-editor-hero">
      <div class="hero-content">
        <h1>{{ deck?.name || "Deck" }}</h1>
        <div class="hero-actions">
          <div class="deck-search-wrap">
            <v-text-field
              v-model="searchName"
              class="deck-search"
              label="Search and add a card"
              density="comfortable"
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
                type="button"
                class="deck-search-suggestion"
                :class="{
                  'deck-search-suggestion--active': index === activeSuggestionIndex,
                  'deck-search-suggestion--illegal':
                    suggestion.commander_legality && suggestion.commander_legality !== 'legal',
                  'deck-search-suggestion--game-changer':
                    suggestion.game_changer &&
                    !(suggestion.commander_legality && suggestion.commander_legality !== 'legal'),
                }"
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
                  <ManaText
                    v-if="suggestion.mana_cost"
                    :text="suggestion.mana_cost"
                    :cost="true"
                  />
                </div>
                <span class="deck-search-suggestion__type">{{ suggestion.type_line }}</span>
              </button>
            </div>
          </div>
          <v-btn
            class="add-card-btn"
            :prepend-icon="mdiPlus"
            :loading="isAddingCard"
            @click="handleAddCard"
          >
            Add Card
          </v-btn>
          <v-btn
            class="package-btn"
            variant="outlined"
            :prepend-icon="mdiPackageVariantClosedPlus"
            :loading="isLoadingPackages"
            @click="openAddPackageToDeckDialog"
          >
            Add Package
          </v-btn>
          <v-btn
            class="import-btn"
            variant="outlined"
            :prepend-icon="mdiDownload"
            @click="openImportDialog"
          >
            Import Deck
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
          <span class="metric-label">Bracket</span>
          <strong>{{ deck.bracket ?? 2 }}</strong>
        </article>
        <article class="metric-card">
          <div class="d-flex align-center">
            <span class="metric-label">Game changers</span>
            <v-icon
              v-if="deck.game_changer_count > 0"
              :icon="mdiGaugeFull"
              size="16"
              color="amber-darken-2"
              class="ml-1"
            ></v-icon>
          </div>
          <strong>{{ deck.game_changer_count ?? 0 }}</strong>
        </article>
        <article v-if="deck.illegal_count > 0" class="metric-card metric-card--illegal">
          <div class="d-flex align-center">
            <span class="metric-label">Illegal</span>
            <v-icon
              :icon="mdiCancel"
              size="16"
              color="error"
              class="ml-1"
            ></v-icon>
          </div>
          <strong>{{ deck.illegal_count }}</strong>
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
                :can-remove-partner="canRemovePartner(entry.card)"
                :can-remove-commander="true"
                :show-add-to-package-action="true"
                @add-copy="handleAddCopy(entry.card.name)"
                @remove-copy="handleDeleteCommander(entry.card.name)"
                @remove-partner="handleRemovePartner(entry.card.id, entry.card.name)"
                @remove-commander="handleRemoveCommander(entry.card.name)"
                @add-to-package="handleAddToPackage(entry.card.name)"
              />
            </div>
            <p v-else class="empty-copy">No commander selected yet.</p>
          </section>
        </aside>

        <main class="deck-main">
          <section class="deck-panel">
            <div class="panel-heading">
              <div class="d-flex align-center flex-grow-1">
                <v-icon :icon="mdiCardsOutline" size="18"></v-icon>
                <h2 class="ml-2">Mainboard</h2>
              </div>
              <div class="d-flex align-center bg-amber-lighten-4 px-2 py-1 rounded-pill" v-if="deck.game_changer_count > 0">
                <v-icon :icon="mdiGaugeFull" size="16" color="amber-darken-3"></v-icon>
                <span class="text-caption font-weight-bold ml-1 text-amber-darken-4">
                  {{ deck.game_changer_count }} GAME CHANGER{{ deck.game_changer_count > 1 ? 'S' : '' }}
                </span>
              </div>
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
                    :editable="true"
                    :can-set-commander="canSetCommander(entry.card)"
                    :can-set-partner="canSetPartner(entry.card)"
                    :show-add-to-package-action="true"
                    @add-copy="handleAddCopy(entry.card.name)"
                    @remove-copy="handleRemoveCopy(entry.card.id, entry.card.name)"
                    @set-commander="handleSetCommander(entry.card.id, entry.card.name)"
                    @set-partner="handleSetPartner(entry.card.id, entry.card.name)"
                    @add-to-package="handleAddToPackage(entry.card.name)"
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

    <!-- Import Deck Dialog -->
    <v-dialog v-model="importDialogVisible" max-width="720">
      <v-card class="import-dialog">
        <v-card-title>Import Deck</v-card-title>
        <v-card-text>
          <p class="import-dialog__help">Paste a deck list using the format: "1 Card Name" per line.</p>
          <v-textarea
            v-model="importText"
            label="Deck list"
            rows="10"
            density="comfortable"
            hide-details="auto"
            class="import-textarea"
            @input="importErrors = validateImportLines(importText)"
          />

          <div v-if="importErrors.length > 0" class="import-errors">
            <p>Format errors found on the following lines:</p>
            <ul>
              <li v-for="err in importErrors" :key="`err-${err.line}`">Line {{ err.line }}: "{{ err.text }}"</li>
            </ul>
          </div>

          <div class="import-actions" style="display:flex;gap:8px;margin-top:12px;align-items:center;">
            <v-btn variant="outlined" :loading="isPasting" @click="pasteFromClipboard">Paste from Clipboard</v-btn>
            <v-btn variant="outlined" @click="importText = ''">Clear List</v-btn>
            <v-spacer />
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="outlined" @click="closeImportDialog">Cancel</v-btn>
          <v-btn
            variant="outlined"
            color="primary"
            :loading="isImporting"
            :disabled="isImporting || importErrors.length > 0"
            @click="handleImport"
          >
            Import
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="packageDialogVisible" max-width="560">
      <v-card class="package-dialog">
        <v-card-title>
          {{ packageDialogMode === "card" ? "Add Card to Package" : "Add Package to Deck" }}
        </v-card-title>
        <v-card-text class="package-dialog__content">
          <p v-if="packageDialogMode === 'card'" class="package-dialog__copy">
            Choose a package for "{{ pendingPackageCardName }}".
          </p>
          <p v-else class="package-dialog__copy">
            Choose a package to add its cards to this deck.
          </p>

          <v-select
            v-model="selectedPackageId"
            :items="packages"
            item-title="name"
            item-value="id"
            label="Package"
            density="comfortable"
            hide-details="auto"
            :loading="isLoadingPackages"
            :disabled="isLoadingPackages"
          />

          <div class="package-dialog__create">
            <v-btn
              variant="outlined"
              :loading="isCreatingPackage"
              @click="handleCreatePackage"
            >
              Create Package
            </v-btn>
            <v-text-field
              v-model="newPackageName"
              label="New package name"
              density="comfortable"
              hide-details="auto"
              @keydown.enter.prevent="handleCreatePackage"
            />
          </div>
        </v-card-text>
        <v-card-actions class="package-dialog__actions">
          <v-spacer />
          <v-btn variant="outlined" @click="closePackageDialog">Cancel</v-btn>
          <v-btn
            variant="outlined"
            color="primary"
            :loading="isSubmittingPackageAction"
            :disabled="packages.length === 0 && !selectedPackageId"
            @click="handleSubmitPackageDialog"
          >
            {{ packageDialogMode === "card" ? "Add to Package" : "Add Package" }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

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
  min-width: 0;
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
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.98) 0%, rgba(247, 250, 255, 0.98) 100%);
  border: 1px solid rgba(27, 42, 63, 0.08);
  box-shadow: 0 22px 40px rgba(20, 31, 48, 0.12);
}

.deck-search-suggestion {
  display: grid;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 14px;
  background: rgba(239, 244, 252, 0.88);
  color: #132032;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s ease;
}

.deck-search-suggestion--illegal {
  background: rgba(254, 242, 242, 0.88);
  border-color: rgba(185, 28, 28, 0.2);
}

.deck-search-suggestion--game-changer {
  background: rgba(255, 247, 237, 0.88);
  border-color: rgba(194, 65, 12, 0.2);
}

.deck-search-suggestion--active,
.deck-search-suggestion:hover {
  background: rgba(217, 229, 246, 0.96);
  border-color: rgba(27, 42, 63, 0.15);
}

.deck-search-suggestion--illegal.deck-search-suggestion--active,
.deck-search-suggestion--illegal:hover {
  background: rgba(254, 226, 226, 0.96);
  border-color: rgba(185, 28, 28, 0.4);
}

.deck-search-suggestion--game-changer.deck-search-suggestion--active,
.deck-search-suggestion--game-changer:hover {
  background: rgba(255, 237, 213, 0.96);
  border-color: rgba(194, 65, 12, 0.4);
}

.deck-search-suggestion__top {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  font-weight: 700;
}

.deck-search-suggestion__type {
  color: #607089;
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

.add-card-btn {
  flex: 0 0 auto;
}

.package-btn {
  flex: 0 0 auto;
}

.import-btn {
  flex: 0 0 auto;
}

.import-textarea {
  max-height: 300px;
  overflow-y: auto !important;
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
  grid-template-columns: repeat(8, minmax(0, 1fr));
  gap: 14px;
  margin-bottom: 20px;
}

.metric-card {
  padding: 16px 18px;
  border-radius: 20px;
  background: #fff;
  border: 1px solid rgba(27, 42, 63, 0.08);
  box-shadow: 0 16px 30px rgba(24, 37, 58, 0.05);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.metric-card--illegal {
  background: #fef2f2;
  border-color: rgba(185, 28, 28, 0.2);
}

.metric-label {
  display: block;
  font-size: 0.82rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #687892;
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

.package-dialog__content {
  display: grid;
  gap: 16px;
}

.package-dialog__copy {
  margin: 0;
  color: #5f6f86;
}

.package-dialog__create {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  align-items: center;
}

.package-dialog__actions {
  padding: 0 24px 20px;
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
  }

  .deck-search-wrap {
    min-width: 0;
  }

  .deck-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .package-dialog__create {
    grid-template-columns: 1fr;
  }
}
</style>
