<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { mdiMagnify } from "@mdi/js";
import { getCardCommand, searchCardSuggestionsCommand } from "../api/deckCommands.js";
import { getBase64ImagesCommand } from "../api/imageCommands.js";
import ManaText from "../components/ManaText.vue";

const selectedCard = ref(null);
const searchName = ref("");
const cardSuggestions = ref([]);
const isSearchingCards = ref(false);
let suggestionSearchTimeout = null;

const typeLine = computed(() => {
  if (!selectedCard.value) return "";
  const superType = Array.isArray(selectedCard.value.super_type) ? selectedCard.value.super_type.join(" ") : "";
  const cardType = Array.isArray(selectedCard.value.card_type) ? selectedCard.value.card_type.join(" ") : "Card";
  const subType = Array.isArray(selectedCard.value.sub_type) && selectedCard.value.sub_type.length > 0 ? ` - ${selectedCard.value.sub_type.join(" ")}` : "";
  return `${superType} ${cardType}${subType}`.trim();
});

const cardImage = computed(() => {
  if (!selectedCard.value || !selectedCard.value.image) return "";
  const image = selectedCard.value.image;
  if (image.startsWith("http") || image.startsWith("data:")) {
    return image;
  }
  return convertFileSrc(image);
});

watch(selectedCard, async (newCard) => {
  if (newCard && newCard.image && !newCard.image.startsWith('data:') && !newCard.image.startsWith('http')) {
    try {
      const b64Array = await getBase64ImagesCommand([newCard.image]);
      if (b64Array && b64Array[0]) {
        newCard.image = b64Array[0];
      }
    } catch (err) {
      console.error("Failed to convert image to base64 in CardViewer:", err);
    }
  }
});

async function handleSearchInput(val) {
  if (!val || val.length < 2) {
    cardSuggestions.value = [];
    return;
  }

  if (suggestionSearchTimeout) clearTimeout(suggestionSearchTimeout);
  
  suggestionSearchTimeout = setTimeout(async () => {
    isSearchingCards.value = true;
    try {
      cardSuggestions.value = await searchCardSuggestionsCommand(val);
    } catch (err) {
      console.error("Search failed:", err);
    } finally {
      isSearchingCards.value = false;
    }
  }, 300);
}

async function selectCard(name) {
  if (!name) return;
  try {
    selectedCard.value = await getCardCommand(name);
    searchName.value = name;
  } catch (err) {
    console.error("Failed to fetch card:", err);
  }
}

let unlistenHover = null;

onMounted(async () => {
  unlistenHover = await listen("hover-card", (event) => {
    selectedCard.value = event.payload;
    if (selectedCard.value) {
      searchName.value = selectedCard.value.name;
    }
  });
});

onUnmounted(() => {
  if (unlistenHover) unlistenHover();
});
</script>

<template>
  <v-container class="card-viewer-container" fluid>
    <v-row>
      <v-col cols="12">
        <v-autocomplete
          v-model="searchName"
          :items="cardSuggestions"
          :loading="isSearchingCards"
          item-title="name"
          item-value="name"
          label="Search Card"
          placeholder="Start typing to search..."
          variant="outlined"
          density="comfortable"
          hide-no-data
          hide-details
          @update:search="handleSearchInput"
          @update:model-value="selectCard"
        >
          <template #prepend-inner>
            <v-icon :icon="mdiMagnify"></v-icon>
          </template>
        </v-autocomplete>
      </v-col>
    </v-row>

    <div v-if="selectedCard" class="mt-4">
      <v-row no-gutters>
        <v-col cols="auto">
          <div class="card-image-column">
            <img v-if="cardImage" :src="cardImage" :alt="selectedCard.name" class="card-img" />
            <div v-else class="card-placeholder-box">No Image Available</div>
          </div>
        </v-col>
        <v-col>
          <div class="card-info-column">
            <div class="d-flex justify-space-between align-start mb-2">
              <h2 class="text-h4 font-weight-bold">{{ selectedCard.name }}</h2>
              <ManaText :text="selectedCard.mana_cost || ''" :cost="true" />
            </div>
            <p class="text-h6 font-italic mb-2">{{ typeLine }}</p>
            <v-divider class="mb-4"></v-divider>
            <p class="text-body-1 oracle-text">{{ selectedCard.oracle_text }}</p>
          </div>
        </v-col>
      </v-row>
    </div>
    <v-row class="mt-4" v-else>
      <v-col cols="12">
        <div class="card-placeholder-box">
          Hover over a card in the deck editor or search for one here.
        </div>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>
.card-viewer-container {
  height: 100vh;
  background-color: rgb(var(--v-theme-surface));
  overflow-y: auto;
}

.card-image-column {
  width: 300px;
  flex: 0 0 300px;
  padding: 0 12px;
}

.card-img {
  width: 100%;
  height: auto;
  border-radius: 4.75% / 3.5%;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.card-placeholder-box {
  height: 420px;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px dashed rgba(var(--v-border-color), 0.1);
  border-radius: 12px;
  padding: 24px;
  text-align: center;
  color: rgba(var(--v-theme-on-surface), 0.5);
}

.card-info-column {
  flex: 1;
  background: rgba(var(--v-border-color), 0.05);
  padding: 24px;
  border-radius: 12px;
  margin-right: 12px;
  max-height: 420px;
  overflow-y: auto;
}

.oracle-text {
    white-space: pre-wrap;
    line-height: 1.4;
}
</style>
