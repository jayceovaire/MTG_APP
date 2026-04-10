<script setup>
import {computed, ref} from "vue";
import {mdiContentCopy, mdiDotsHorizontal, mdiPencil, mdiTrashCan} from "@mdi/js";
import {deleteDeckCommand, duplicateDeckCommand, setDeckNameCommand} from "../api/deckCommands.js";
import ManaText from "./ManaText.vue";

const props = defineProps({
  deck: {
    type: Object,
    required: true,
  },
});
const emit = defineEmits(["deck-renamed", "deck-deleted", "deck-duplicated", "deck-action-error"]);

const renameDialogOpen = ref(false);
const actionsMenuOpen = ref(false);
const pendingName = ref("");
const isRenaming = ref(false);

function commanderCards(commander) {
  if (!commander || commander === "None") {
    return [];
  }

  if (commander.Single) {
    return [commander.Single];
  }

  return Array.isArray(commander.Partner) ? commander.Partner : [];
}

const deckName = computed(() => props.deck?.name || "Untitled");
const commanderName = computed(() => {
  const commander = props.deck?.commander;
  if (!commander || commander === "None") {
    return "No Commander";
  }

  if (commander.Single?.name) {
    return commander.Single.name;
  }

  if (Array.isArray(commander.Partner) && commander.Partner.length === 2) {
    return commander.Partner
      .map((card) => card?.name)
      .filter(Boolean)
      .join(" / ");
  }

  return "No Commander";
});
const allDeckCards = computed(() => {
  const cards = Array.isArray(props.deck?.cards) ? props.deck.cards : [];
  return [...cards, ...commanderCards(props.deck?.commander)];
});
const cardCount = computed(() => allDeckCards.value.length);
const colorProfile = computed(() => {
  const cards = allDeckCards.value;
  const colorOrder = ["W", "U", "B", "R", "G"];
  const colors = new Set();

  function addColorsFromCard(card) {
    const manaCost = typeof card?.mana_cost === "string" ? card.mana_cost : "";
    for (const color of colorOrder) {
      if (manaCost.includes(`{${color}}`)) {
        colors.add(color);
      }
    }
  }

  for (const card of cards) {
    addColorsFromCard(card);
  }

  return colorOrder.filter((color) => colors.has(color)).join("") || "Colorless";
});
const colorProfileSymbols = computed(() => {
  if (colorProfile.value === "Colorless") {
    return "";
  }

  return [...colorProfile.value].map((color) => `{${color}}`).join("");
});
const averageManaValue = computed(() => {
  const cards = allDeckCards.value;
  if (cards.length === 0) {
    return "0.00";
  }

  const totalManaValue = cards.reduce((sum, card) => sum + (Number(card?.mana_value) || 0), 0);
  return (totalManaValue / cards.length).toFixed(2);
});
const bracket = computed(() => props.deck?.bracket || "2");

// RENAME DECK
function openRenameDialog() {
  actionsMenuOpen.value = false;
  pendingName.value = deckName.value;
  renameDialogOpen.value = true;
}

async function submitRename() {
  if (isRenaming.value) {
    return;
  }

  try {
    isRenaming.value = true;
    const renamedDeck = await setDeckNameCommand(props.deck, pendingName.value);
    emit("deck-renamed", renamedDeck);
    renameDialogOpen.value = false;
  } catch (e) {
    emit("deck-action-error", `Failed to rename deck: ${String(e)}`);
    console.error(e);
  } finally {
    isRenaming.value = false;
  }
}

// DUPLICATE DECK
async function submitDuplicate() {
  try {
    actionsMenuOpen.value = false;
    const duplicatedDeck = await duplicateDeckCommand(props.deck.id);
    emit("deck-duplicated", duplicatedDeck);
  } catch (e) {
    emit("deck-action-error", `Failed to duplicate deck: ${String(e)}`);
    console.error(e);
  }
}

//DELETE DECK
async function submitDelete(){
  try {
    actionsMenuOpen.value = false;
    await deleteDeckCommand(props.deck.id);
    emit("deck-deleted", props.deck.id);
  } catch (e) {
    emit("deck-action-error", `Failed to delete deck: ${String(e)}`);
    console.error(e);
  }
}


</script>

<template>
  <v-card
    variant="flat"
    border
    class="deck-tile pa-4 h-100 deck-card-hover"
  >
    <div class="d-flex align-center mb-3">
      <div class="overflow-hidden">
        <h3 class="text-h6 font-weight-bold text-truncate text-left text-primary">{{ deckName }}</h3>
        <div class="text-caption text-medium-emphasis text-left">{{ cardCount }} Cards</div>
      </div>
      <v-spacer />
      <v-menu v-model="actionsMenuOpen" location="bottom end">
        <template #activator="{ props: menuProps }">
          <v-btn v-bind="menuProps" @click.stop variant="text" icon size="small" class="mt-n4 mr-n2">
            <v-icon :icon="mdiDotsHorizontal"></v-icon>
          </v-btn>
        </template>
        <v-list density="compact">
          <v-list-item @click.stop="openRenameDialog">
            <template #prepend>
              <v-icon :icon="mdiPencil" size="18"></v-icon>
            </template>
            <v-list-item-title>Rename</v-list-item-title>
          </v-list-item>
          <v-list-item @click.stop="submitDuplicate">
            <template #prepend>
              <v-icon :icon="mdiContentCopy" size="18"></v-icon>
            </template>
            <v-list-item-title>Duplicate</v-list-item-title>
          </v-list-item>
          <v-divider />
          <v-list-item @click.stop="submitDelete" class="text-error">
            <template #prepend>
              <v-icon :icon="mdiTrashCan" size="18" color="error"></v-icon>
            </template>
            <v-list-item-title>Delete</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
    </div>

    <div class="text-body-2 text-medium-emphasis mb-4 text-truncate text-left">
      {{ commanderName }}
    </div>

    <v-divider class="mb-4" />

    <div class="deck-stats">
      <div class="stat-row">
        <span class="text-caption text-medium-emphasis">Color</span>
        <ManaText
          class="stat-value"
          :text="colorProfileSymbols"
          :empty-text="colorProfile"
          :cost="true"
        />
      </div>
      <div class="stat-row">
        <span class="text-caption text-medium-emphasis">Avg MV</span>
        <span class="stat-value">{{ averageManaValue }}</span>
      </div>
      <div class="stat-row">
        <span class="text-caption text-medium-emphasis">Bracket</span>
        <span class="stat-value">{{ bracket }}</span>
      </div>
    </div>

    <v-dialog v-model="renameDialogOpen" max-width="420">
      <v-card @click.stop>
        <v-card-title>Rename Deck</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="pendingName"
            label="Deck Name"
            autofocus
            hide-details
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="renameDialogOpen = false">Cancel</v-btn>
          <v-btn :loading="isRenaming" @click="submitRename">Change Name</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-card>
</template>

<style scoped>
.deck-tile {
  cursor: pointer;
  transition: transform 0.2s, border-color 0.2s;
}

.deck-tile:hover {
  transform: translateY(-4px);
}

.deck-card-hover:hover {
  border-color: rgb(var(--v-theme-primary)) !important;
  background-color: rgba(var(--v-theme-primary), 0.02) !important;
}

.gap-4 {
  gap: 16px;
}

.deck-stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-value {
  font-weight: 600;
}
</style>
