<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import {
  mdiAlertCircleOutline,
  mdiCardsOutline,
  mdiFilterVariant,
  mdiHeart,
  mdiPlus,
  mdiRefresh,
  mdiCancel,
  mdiGaugeFull,
} from "@mdi/js";
import {
  createCollectionCardCommand,
  duplicateCollectionCardCommand,
  getCollectionCommand,
  removeCollectionCardCommand,
  setCollectionCardFavoriteCommand,
} from "../api/collectionCommands.js";
import { searchCardSuggestionsCommand } from "../api/deckCommands.js";
import DeckCardRow from "../components/DeckCardRow.vue";
import ManaText from "../components/ManaText.vue";

const cards = ref([]);
const isLoading = ref(false);
const isCreatingCard = ref(false);
const newCardName = ref("");
const cardSuggestions = ref([]);
const isSearchingCards = ref(false);
const isSearchFocused = ref(false);
const activeSuggestionIndex = ref(-1);
const loadError = ref("");
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");
const snackbarTimeout = 2500;
const activeFilters = ref([]);
let suggestionSearchTimeout = null;
let searchBlurTimeout = null;
let unlistenImages = null;

const typeFilterOptions = [
  "Creature",
  "Artifact",
  "Enchantment",
  "Instant",
  "Sorcery",
  "Planeswalker",
  "Battle",
  "Land",
  "Tribal",
];
const favoritesFilterKey = "Favorites";

async function loadCollection() {
  isLoading.value = true;
  loadError.value = "";

  try {
    cards.value = await getCollectionCommand();
  } catch (e) {
    loadError.value = `Failed to load collection: ${String(e)}`;
    showError(loadError.value);
    console.error(e);
  } finally {
    isLoading.value = false;
  }
}

onMounted(async () => {
  await loadCollection();
  unlistenImages = await listen("images-updated", () => {
    console.log("Images updated event received in Collection, reloading...");
    loadCollection();
  });
});

onUnmounted(() => {
  if (unlistenImages) {
    unlistenImages();
  }
});

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
  newCardName.value = suggestion.name;
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

  await handleCreateCard();
}

async function handleCreateCard() {
  if (isCreatingCard.value) {
    return;
  }

  const trimmed = newCardName.value.trim();
  if (!trimmed) {
    showError("Enter a card name first.");
    return;
  }

  //TODO In the future this will look for a card in the database from the backend and then push it, not just make a new card
  try {
    isCreatingCard.value = true;
    const newCard = await createCollectionCardCommand(trimmed);
    cards.value.push(newCard);
    showSuccess(`Added "${newCard.name}" to collection`);
    newCardName.value = "";
    clearSuggestions();
  } catch (e) {
    showError(`Failed to add card: ${String(e)}`);
    console.error(e);
  } finally {
    isCreatingCard.value = false;
  }
}

async function handleCardRemoved(cardId) {
  try {
    await removeCollectionCardCommand(cardId);
    cards.value = cards.value.filter((card) => card.id !== cardId);
    showSuccess("Removed card from collection");
  } catch (e) {
    showError(`Failed to remove card: ${String(e)}`);
    console.error(e);
  }
}

async function handleCardAdded(card) {
  try {
    const duplicatedCard = await duplicateCollectionCardCommand(card.id);
    cards.value.push(duplicatedCard);
    showSuccess(`Added another "${duplicatedCard.name}"`);
  } catch (e) {
    showError(`Failed to duplicate card: ${String(e)}`);
    console.error(e);
  }
}

async function handleCardFavorited(card) {
  try {
    const updatedCard = await setCollectionCardFavoriteCommand(card.id, !card.favorite);
    const index = cards.value.findIndex((entry) => entry.id === card.id);
    if (index >= 0) {
      cards.value[index] = updatedCard;
    }
    showSuccess(updatedCard.favorite ? `Favorited "${card.name}"` : `Unfavorited "${card.name}"`);
  } catch (e) {
    showError(`Failed to update favorite: ${String(e)}`);
    console.error(e);
  }
}

function collapseCardCopies(cardsToCollapse) {
  const grouped = new Map();

  for (const card of cardsToCollapse) {
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
      entry.cards.push(card);
      entry.favorited = entry.favorited || Boolean(card.favorite);
      if (card.favorite && !entry.card.favorite) {
        entry.card = card;
      }
      continue;
    }

    grouped.set(key, {
      card,
      quantity: 1,
      cards: [card],
      favorited: Boolean(card.favorite),
    });
  }

  return [...grouped.values()].sort((a, b) => a.card.name.localeCompare(b.card.name));
}

const showCardSuggestions = computed(() => (
  isSearchFocused.value &&
  cardSuggestions.value.length > 0 &&
  newCardName.value.trim().length >= 2
));
const collectionCount = computed(() => cards.value.length);
const activeFilterCount = computed(() => activeFilters.value.length);
const groupedCollection = computed(() => {
  const favoritesOnly = activeFilters.value.includes(favoritesFilterKey);
  const selectedTypes = activeFilters.value.filter((filter) => filter !== favoritesFilterKey);

  return collapseCardCopies(cards.value).filter((entry) => {
    if (favoritesOnly && !entry.favorited) {
      return false;
    }

    if (selectedTypes.length === 0) {
      return true;
    }

    const cardTypes = Array.isArray(entry.card.card_type) ? entry.card.card_type : [];
    return selectedTypes.every((type) => cardTypes.includes(type));
  });
});

function toggleFilter(filter) {
  if (activeFilters.value.includes(filter)) {
    activeFilters.value = activeFilters.value.filter((entry) => entry !== filter);
    return;
  }

  activeFilters.value = [...activeFilters.value, filter];
}

function clearFilters() {
  activeFilters.value = [];
}

watch(newCardName, (value) => {
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

</script>

<template>
  <v-container class="collection-page">
    <div class="d-flex align-center justify-space-between mb-6">
      <h1 class="text-h4 font-weight-bold text-primary">Collection</h1>
      <div class="d-flex align-center gap-3">
        <v-menu location="bottom end">
          <template #activator="{ props: menuProps }">
            <v-btn
              v-bind="menuProps"
              variant="outlined"
              :prepend-icon="mdiFilterVariant"
              class="text-none"
              rounded="lg"
            >
              Filter<span v-if="activeFilterCount > 0" class="ml-1">({{ activeFilterCount }})</span>
            </v-btn>
          </template>
          <v-list density="compact" class="filter-menu">
            <v-list-item
              :active="activeFilters.includes(favoritesFilterKey)"
              title="Favorites"
              @click.stop="toggleFilter(favoritesFilterKey)"
            >
              <template #prepend>
                <v-icon :icon="mdiHeart" size="16"></v-icon>
              </template>
            </v-list-item>
            <v-divider class="my-1"></v-divider>
            <v-list-item
              v-for="filter in typeFilterOptions"
              :key="filter"
              :active="activeFilters.includes(filter)"
              :title="filter"
              @click.stop="toggleFilter(filter)"
            />
            <v-divider class="my-1"></v-divider>
            <v-list-item title="Clear Filters" @click.stop="clearFilters" />
          </v-list>
        </v-menu>

        <v-btn 
          color="primary" 
          :loading="isCreatingCard" 
          @click="handleCreateCard"
          class="text-none font-weight-bold"
          rounded="lg"
        >
          <template #prepend>
            <div class="d-flex align-center mr-1">
              <v-icon :icon="mdiPlus" size="18"></v-icon>
              <v-icon :icon="mdiCardsOutline" size="18" class="ml-n1"></v-icon>
            </div>
          </template>
          Add Card
        </v-btn>
      </div>
    </div>

    <v-divider class="mb-6"></v-divider>

    <v-row class="mb-6">
      <v-col cols="12" md="6" lg="5">
        <div class="card-name-input">
          <v-text-field
            v-model="newCardName"
            class="collection-search"
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
          >
            <template #prepend-inner>
              <v-icon :icon="mdiCardsOutline" color="primary" class="mr-2"></v-icon>
            </template>
          </v-text-field>

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
        </v-col>
      </v-row>

    <div v-if="loadError" class="feedback feedback--error mb-6">
      <v-icon :icon="mdiAlertCircleOutline" size="18"></v-icon>
      <span>{{ loadError }}</span>
    </div>

    <v-card variant="flat" border class="pa-6">
      <div class="d-flex align-center justify-space-between mb-6">
        <div class="d-flex align-center">
          <v-icon :icon="mdiCardsOutline" color="primary" class="mr-3" size="28"></v-icon>
          <h2 class="text-h5 font-weight-bold">All Cards</h2>
        </div>
        <v-chip color="primary" variant="tonal" size="small" class="font-weight-bold">
          {{ collectionCount }} cards
        </v-chip>
      </div>

      <v-divider class="mb-4"></v-divider>

      <div v-if="groupedCollection.length > 0" class="deck-list">
        <DeckCardRow
          v-for="entry in groupedCollection"
          :key="`collection-${entry.card.id}`"
          :card="entry.card"
          :quantity="entry.quantity"
          :editable="true"
          :favorited="entry.favorited"
          :show-favorite-indicator="true"
          :show-favorite-action="true"
          @add-copy="handleCardAdded(entry.card)"
          @remove-copy="handleCardRemoved(entry.cards[0].id)"
          @favorite-card="handleCardFavorited(entry.card)"
        />
      </div>

      <div v-else class="text-center pa-12">
        <v-icon :icon="mdiCardsOutline" size="64" class="mb-4 opacity-20"></v-icon>
        <h3 class="text-h6 text-medium-emphasis">Your collection is empty</h3>
        <p class="text-body-2 text-medium-emphasis">Search for a card above and add it to start building out your library.</p>
      </div>
    </v-card>

    <v-snackbar
        v-model="snackbarVisible"
        :color="snackbarColor"
        :timeout="snackbarTimeout"
        rounded="pill"
    >
      {{ snackbarMessage }}
    </v-snackbar>
  </v-container>
</template>

<style scoped>
.collection-page {
}

.gap-3 {
  gap: 12px;
}

.search-card {
  border-radius: 12px !important;
}

.card-name-input {
  position: relative;
  min-width: 320px;
  max-width: 520px;
  width: 100%;
}

.deck-search-suggestions {
  position: absolute;
  top: calc(100% + 12px);
  left: -8px;
  right: -8px;
  z-index: 20;
  display: grid;
  gap: 6px;
  padding: 10px;
  border-radius: 18px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(var(--v-border-color), 0.08);
  box-shadow: 0 22px 40px rgba(0, 0, 0, 0.4);
}

.deck-search-suggestion {
  display: grid;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 14px;
  background: rgba(var(--v-border-color), 0.05);
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s ease;
}

.deck-search-suggestion--illegal {
  background: rgba(185, 28, 28, 0.1);
  border-color: rgba(185, 28, 28, 0.2);
}

.deck-search-suggestion--game-changer {
  background: rgba(194, 65, 12, 0.1);
  border-color: rgba(194, 65, 12, 0.2);
}

.deck-search-suggestion--active,
.deck-search-suggestion:hover {
  background: rgba(var(--v-theme-primary), 0.1);
  border-color: rgba(var(--v-theme-primary), 0.3);
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

.deck-list {
  display: grid;
  gap: 12px;
}

.feedback {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  border-radius: 12px;
}

.feedback--error {
  background: rgba(var(--v-theme-error), 0.1);
  color: rgb(var(--v-theme-error));
}

.opacity-20 {
  opacity: 0.2;
}
</style>
