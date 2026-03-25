<script setup>
import {computed, ref} from "vue";
import {useRouter} from "vue-router";
import {mdiContentCopy, mdiDotsHorizontal, mdiPencil, mdiTrashCan} from "@mdi/js";
import {deleteDeckCommand, duplicateDeckCommand, setDeckNameCommand} from "../api/deckCommands.js";
import ManaText from "./ManaText.vue";

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
    <v-card
      @click="goToDeckEditor"
      class="deck-tile"
      width="250"
      height="250"
      max-height="500"
      max-width="500"
    >
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
          <ManaText
            class="stat-value stat-value--colors"
            :text="colorProfileSymbols"
            :empty-text="colorProfile"
            :cost="true"
          />
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
  position: relative;
  overflow: hidden;
  border-radius: 24px;
  background:
    radial-gradient(circle at top left, rgba(187, 214, 255, 0.5), transparent 34%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.82) 0%, rgba(247, 250, 255, 0.88) 100%);
  border: 1px solid rgba(27, 42, 63, 0.08);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.78),
    inset 0 -1px 0 rgba(255, 255, 255, 0.2),
    0 20px 40px rgba(20, 31, 48, 0.08);
  backdrop-filter: blur(14px) saturate(120%);
  isolation: isolate;
  transition:
    transform 160ms ease,
    box-shadow 160ms ease,
    border-color 160ms ease;
}

.deck-tile:hover {
  transform: translateY(-6px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.82),
    inset 0 -1px 0 rgba(255, 255, 255, 0.24),
    0 28px 46px rgba(20, 31, 48, 0.14);
  border-color: rgba(27, 42, 63, 0.12);
}

.deck-tile::before,
.deck-tile::after {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.deck-tile::before {
  background:
    linear-gradient(135deg, rgba(255, 255, 255, 0.38) 0%, rgba(255, 255, 255, 0.16) 22%, rgba(255, 255, 255, 0.02) 46%),
    radial-gradient(circle at top left, rgba(255, 255, 255, 0.55) 0%, rgba(255, 255, 255, 0) 34%);
  opacity: 0.9;
}

.deck-tile::after {
  inset: auto 10px 10px 10px;
  height: 44%;
  border-radius: 18px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.18) 0%, rgba(255, 255, 255, 0.03) 100%);
  opacity: 0.7;
}

.tile-header {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  position: relative;
  z-index: 1;
}

.tile-item {
  margin: 0;
  color: #132032;
}

.tile-button {
  min-width: 24px !important;
  width: 24px;
  height: 24px;
  padding: 0;
  background: rgba(255, 255, 255, 0.8);
  color: #132032;
}

.commander-name {
  margin-top: 8px;
  color: #5f6f86;
  position: relative;
  z-index: 1;
}

.tile-footer {
  width: 100%;
  margin-top: auto;
  padding-top: 12px;
  display: grid;
  gap: 6px;
  position: relative;
  z-index: 1;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  line-height: 1.1;
}

.stat-label {
  color: #687892;
}

.stat-value {
  font-weight: 600;
  color: #132032;
}

.stat-value--colors {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-height: 20px;
}
</style>
