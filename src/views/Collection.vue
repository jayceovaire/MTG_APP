<script setup>
import { computed, onMounted, ref, watch } from "vue";
import {
  mdiAlertCircleOutline,
  mdiCardsOutline,
  mdiFilterVariant,
  mdiHeart,
  mdiPlus,
  mdiRefresh,
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

onMounted(loadCollection);

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

  //TODO In the future this will look for a card in the database from the backend and then push it, not just make a new card
  try {
    isCreatingCard.value = true;
    const newCard = await createCollectionCardCommand(newCardName.value);
    cards.value.push(newCard);
    showSuccess(`Added "${newCard.name}" to collection`);
    newCardName.value = "";
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
    <section class="collection-hero">
      <div class="hero-content">
        <h1>Collection</h1>
        <div class="hero-actions">
          <div class="card-name-input">
            <v-text-field
              v-model="newCardName"
              class="collection-search"
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
                :class="{ 'deck-search-suggestion--active': index === activeSuggestionIndex }"
                @mousedown.prevent="selectSuggestion(suggestion)"
              >
                <div class="deck-search-suggestion__top">
                  <span>{{ suggestion.name }}</span>
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

          <v-btn class="create-card-btn" :loading="isCreatingCard" @click="handleCreateCard">
            <template #prepend>
              <span class="create-card-icons" aria-hidden="true">
                <v-icon :icon="mdiPlus" size="16"></v-icon>
                <v-icon :icon="mdiCardsOutline" size="16"></v-icon>
              </span>
            </template>
            Add Card
          </v-btn>
          <v-menu location="bottom end">
            <template #activator="{ props: menuProps }">
              <v-btn
                v-bind="menuProps"
                class="filter-btn"
                variant="outlined"
                :prepend-icon="mdiFilterVariant"
              >
                Filter<span v-if="activeFilterCount > 0"> ({{ activeFilterCount }})</span>
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
        </div>
      </div>
    </section>

    <div v-if="loadError" class="feedback feedback--error">
      <v-icon :icon="mdiAlertCircleOutline" size="18"></v-icon>
      <span>{{ loadError }}</span>
    </div>

    <section class="collection-panel">
      <div class="panel-heading">
        <v-icon :icon="mdiCardsOutline" size="18"></v-icon>
        <h2>All Cards</h2>
        <span class="panel-count">{{ collectionCount }} cards</span>
      </div>

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

      <div v-else class="empty-state">
        <h3>Your collection is empty</h3>
        <p>Search for a card above and add it to start building out your library.</p>
      </div>
    </section>

    <v-snackbar
        v-model="snackbarVisible"
        :color="snackbarColor"
        :timeout="snackbarTimeout"
    >
      {{ snackbarMessage }}
    </v-snackbar>
  </v-container>
</template>

<style scoped>
.collection-page {
  max-width: 1440px;
  padding: 28px;
  color: #132032;
}

.collection-hero {
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

.collection-hero h1 {
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

.card-name-input {
  position: relative;
  min-width: 320px;
  max-width: 520px;
  width: 100%;
}

.collection-search {
  min-width: 0;
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
  border: 0;
  border-radius: 14px;
  background: rgba(239, 244, 252, 0.88);
  color: #132032;
  text-align: left;
  cursor: pointer;
}

.deck-search-suggestion--active,
.deck-search-suggestion:hover {
  background: rgba(217, 229, 246, 0.96);
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

.create-card-btn {
  flex: 0 0 auto;
  text-transform: none;
  font-weight: 600;
}

.refresh-btn {
  flex: 0 0 auto;
}

.create-card-icons {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.collection-panel {
  padding: 20px;
  border-radius: 24px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.96) 0%, rgba(247, 250, 255, 0.98) 100%);
  border: 1px solid rgba(27, 42, 63, 0.08);
  box-shadow: 0 20px 40px rgba(20, 31, 48, 0.05);
}

.deck-list {
  display: grid;
  gap: 10px;
}

.panel-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.panel-heading h2 {
  margin: 0;
}

.panel-count {
  margin-left: auto;
  color: #607089;
  font-size: 0.9rem;
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

.empty-state {
  padding: 36px 12px 18px;
  text-align: center;
}

.empty-state h3 {
  margin: 0 0 8px;
}

.empty-state p {
  margin: 0;
  color: #5f6f86;
  line-height: 1.5;
}

@media (max-width: 720px) {
  .collection-page {
    padding: 16px;
  }

  .collection-hero {
    padding: 22px;
  }

  .hero-actions {
    width: 100%;
    justify-content: stretch;
  }

  .card-name-input {
    min-width: 0;
  }

  .collection-search {
    width: 100%;
  }
}

</style>
