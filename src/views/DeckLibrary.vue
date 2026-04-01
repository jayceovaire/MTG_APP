<script setup>

import DeckTile from "../components/DeckTile.vue";
import {onMounted, ref} from "vue";
import {createDeckCommand, getDecksCommand} from "../api/deckCommands.js";
import {mdiCardsOutline, mdiPlus} from "@mdi/js";

const decks = ref([]);
const isCreatingDeck = ref(false);
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");
const snackbarTimeout = 2500;

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

onMounted(async () => {
  decks.value = await getDecksCommand();
  console.log("completed getDecks")
});

function handleDeckRenamed(updatedDeck) {
  const deckIndex = decks.value.findIndex((deck) => deck.id === updatedDeck.id);
  if (deckIndex === -1) {
    return;
  }

  decks.value[deckIndex] = updatedDeck;
  showSuccess(`Renamed deck to "${updatedDeck.name}"`);
}

async function handleCreateDeck() {
  if (isCreatingDeck.value) {
    return;
  }

  try {
    isCreatingDeck.value = true;
    const newDeck = await createDeckCommand("Untitled");
    decks.value.push(newDeck);
    showSuccess(`Created deck "${newDeck.name}"`);
  } catch (e) {
    showError(`Failed to create deck: ${String(e)}`);
    console.error(e);
  } finally {
    isCreatingDeck.value = false;
  }
}

function handleDeckDeleted(deletedId){
  decks.value = decks.value.filter(deck => deck.id !== deletedId);
  showSuccess("Deck deleted");
}

function handleDeckDuplicated(duplicatedDeck) {
  decks.value.push(duplicatedDeck);
  showSuccess(`Duplicated deck as "${duplicatedDeck.name}"`);
}

function handleDeckActionError(message) {
  showError(message);
}

</script>

<template>
  <v-container class="deck-library-view">
    <div class="d-flex align-center justify-space-between mb-6">
      <h1 class="text-h4 font-weight-bold">Deck Library</h1>
      <v-btn 
        color="primary" 
        :loading="isCreatingDeck" 
        @click="handleCreateDeck"
        class="text-none font-weight-bold"
        rounded="lg"
      >
        <template #prepend>
          <div class="d-flex align-center mr-1">
            <v-icon :icon="mdiPlus" size="18"></v-icon>
            <v-icon :icon="mdiCardsOutline" size="18" class="ml-n1"></v-icon>
          </div>
        </template>
        Create Deck
      </v-btn>
    </div>

    <v-divider class="mb-6"></v-divider>

    <div v-if="decks.length === 0" class="d-flex flex-column align-center justify-center pa-12 text-medium-emphasis">
      <v-icon :icon="mdiCardsOutline" size="64" class="mb-4 opacity-20"></v-icon>
      <div class="text-h6">No decks found</div>
      <p>Create a new deck to get started.</p>
    </div>

    <div v-else class="deck-grid">
      <DeckTile
          v-for="deck in decks"
          :key="deck.id"
          :deck="deck"
          @deck-renamed="handleDeckRenamed"
          @deck-deleted="handleDeckDeleted"
          @deck-duplicated="handleDeckDuplicated"
          @deck-action-error="handleDeckActionError"
      />
    </div>

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
.deck-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}
</style>
