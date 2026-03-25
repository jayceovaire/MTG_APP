<script setup>
import {computed, ref} from "vue";
import {useRouter} from "vue-router";
import {mdiContentCopy, mdiDotsHorizontal, mdiPencil, mdiTrashCan} from "@mdi/js";
import {deleteDeckCommand, duplicateDeckCommand, setDeckNameCommand} from "../api/deckCommands.js";

const router = useRouter();
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
const cardCount = computed(() => Array.isArray(props.deck?.cards) ? props.deck.cards.length : 0);
const colorProfile = computed(() => {
  const cards = Array.isArray(props.deck?.cards) ? props.deck.cards : [];
  const commander = props.deck?.commander;
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

  if (commander?.Single) {
    addColorsFromCard(commander.Single);
  } else if (Array.isArray(commander?.Partner)) {
    for (const card of commander.Partner) {
      addColorsFromCard(card);
    }
  }

  return colorOrder.filter((color) => colors.has(color)).join("") || "Colorless";
});
const averageManaValue = computed(() => {
  const cards = Array.isArray(props.deck?.cards) ? props.deck.cards : [];
  if (cards.length === 0) {
    return "0.00";
  }

  const totalManaValue = cards.reduce((sum, card) => sum + (Number(card?.mana_value) || 0), 0);
  return (totalManaValue / cards.length).toFixed(2);
});
const bracket = computed(() => "4");

function goToDeckEditor() {
  router.push(`/deck-editor/${props.deck.id}`);
}

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
  <v-container>
    <v-card @click="goToDeckEditor" class="deck-tile" width="250" height="250" max-height="500" max-width="500">
      <div class="tile-header">
        <h3 class="tile-item">{{ deckName }}</h3>
        <v-menu v-model="actionsMenuOpen" location="bottom">
          <template #activator="{ props: menuProps }">
            <v-btn v-bind="menuProps" @click.stop size="small" class="tile-button" height="25" width="25">
              <v-icon :icon="mdiDotsHorizontal" size="16"></v-icon>
            </v-btn>
          </template>
          <v-list density="compact">
            <v-list-item @click.stop="openRenameDialog" title="Rename Deck">
              <template #prepend>
                <v-icon :icon="mdiPencil"></v-icon>
              </template>
            </v-list-item>
            <v-list-item @click.stop="submitDuplicate" title="Duplicate Deck">
              <template #prepend>
                <v-icon :icon="mdiContentCopy"></v-icon>
              </template>
            </v-list-item>
            <v-list-item @click.stop="submitDelete" title="Delete Deck">
              <template #prepend>
                <v-icon :icon="mdiTrashCan"></v-icon>
              </template>
            </v-list-item>
          </v-list>
        </v-menu>
      </div>
      <div class="commander-name">{{ commanderName }}</div>
      <div class="tile-footer">
        <div class="stat-row">
          <span class="stat-label">Cards</span>
          <span class="stat-value">{{ cardCount }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">Color</span>
          <span class="stat-value">{{ colorProfile }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">Avg MV</span>
          <span class="stat-value">{{ averageManaValue }}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">Bracket</span>
          <span class="stat-value">{{ bracket }}</span>
        </div>
      </div>
    </v-card>

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
  </v-container>
</template>

<style scoped>
.deck-tile {
  padding: 15px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.tile-header {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.tile-item {
  margin: 0;
}

.tile-button {
  min-width: 24px !important;
  width: 24px;
  height: 24px;
  padding: 0;
}

.commander-name {
  margin-top: 8px;
}

.tile-footer {
  width: 100%;
  margin-top: auto;
  padding-top: 12px;
  display: grid;
  gap: 6px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  line-height: 1.1;
}

.stat-label {
  opacity: 0.75;
}

.stat-value {
  font-weight: 600;
}
</style>
