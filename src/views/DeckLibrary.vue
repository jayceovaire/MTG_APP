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
  <v-container>
    <div class="deck-library-header">
      <v-btn class="create-deck-btn" :loading="isCreatingDeck" @click="handleCreateDeck">
        <template #prepend>
          <span class="create-deck-icons" aria-hidden="true">
            <v-icon :icon="mdiPlus" size="16"></v-icon>
            <v-icon :icon="mdiCardsOutline" size="16"></v-icon>
          </span>
        </template>
        Create Deck
      </v-btn>
    </div>

    <div class="deck-grid">
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
    >
      {{ snackbarMessage }}
    </v-snackbar>
  </v-container>

</template>

<style scoped>
.deck-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 250px));
  justify-content: start;
  gap: 16px;
}

.deck-library-header {
  display: flex;
  justify-content: flex-start;
  margin-bottom: 16px;
}

.create-deck-btn {
  text-transform: none;
  font-weight: 600;
}

.create-deck-icons {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}
</style>
