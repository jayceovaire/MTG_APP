<script setup>
import { onMounted, ref } from "vue";
import { mdiCardsOutline, mdiPlus } from "@mdi/js";
import {
  createCollectionCardCommand,
  duplicateCollectionCardCommand,
  getCollectionCommand,
  removeCollectionCardCommand,
} from "../api/collectionCommands.js";
import CardTile from "../components/CardTile.vue";

const cards = ref([]);
const isCreatingCard = ref(false);
const newCardName = ref("");
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");
const snackbarTimeout = 2500;

onMounted(async () => {
  try {
    cards.value = await getCollectionCommand();
  } catch (e) {
    showError(`Failed to load collection: ${String(e)}`);
    console.error(e);
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

function buildTypeLine(card) {
  const superType = Array.isArray(card.super_type) ? card.super_type.join(" ") : "";
  const cardType = Array.isArray(card.card_type) ? card.card_type.join(" ") : "Card";
  const subType = Array.isArray(card.sub_type) && card.sub_type.length > 0 ? ` - ${card.sub_type.join(" ")}` : "";
  return `${superType} ${cardType}${subType}`.trim();
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

function handleCardFavorited(card) {
  card.favorite = !card.favorite;
  showSuccess(card.favorite ? `Favorited "${card.name}"` : `Unfavorited "${card.name}"`);
}

</script>

<template>
  <v-container>
    <div class="collection-header">
      <v-text-field
        v-model="newCardName"
        class="card-name-input"
        label="Card Name"
        density="comfortable"
        hide-details
        @keyup.enter="handleCreateCard"
      />
      <v-btn class="create-card-btn" :loading="isCreatingCard" @click="handleCreateCard">
        <template #prepend>
          <span class="create-card-icons" aria-hidden="true">
            <v-icon :icon="mdiPlus" size="16"></v-icon>
            <v-icon :icon="mdiCardsOutline" size="16"></v-icon>
          </span>
        </template>
        Add Card
      </v-btn>
    </div>

    <div class="card-grid">
      <CardTile
        v-for="card in cards"
        :key="card.id"
        :name="card.name"
        :mana-cost="card.mana_cost || ''"
        :type-line="buildTypeLine(card)"
        :oracle-text="card.oracle_text || ''"
        :image="card.image || ''"
        :favorited="Boolean(card.favorite)"
        @remove-card="handleCardRemoved(card.id)"
        @add-card="handleCardAdded(card)"
        @favorite-card="handleCardFavorited(card)"
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
.collection-header {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.card-name-input {
  max-width: 320px;
}

.create-card-btn {
  text-transform: none;
  font-weight: 600;
}

.create-card-icons {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 250px));
  justify-content: start;
  gap: 16px;
}

</style>
