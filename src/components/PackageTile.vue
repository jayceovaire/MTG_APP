<script setup>
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { mdiContentCopy, mdiDotsHorizontal, mdiPencil, mdiTrashCan } from "@mdi/js";
import {
  deletePackageCommand,
  duplicatePackageCommand,
  setPackageDescriptionCommand,
  setPackageNameCommand,
} from "../api/deckCommands.js";
import ManaText from "./ManaText.vue";

const router = useRouter();
const props = defineProps({
  packageEntry: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits([
  "package-renamed",
  "package-description-updated",
  "package-deleted",
  "package-duplicated",
  "package-action-error",
]);

const renameDialogOpen = ref(false);
const descriptionDialogOpen = ref(false);
const actionsMenuOpen = ref(false);
const pendingName = ref("");
const pendingDescription = ref("");
const isRenaming = ref(false);
const isSavingDescription = ref(false);

const packageName = computed(() => props.packageEntry?.name || "Untitled Package");
const packageDescription = computed(() => props.packageEntry?.description?.trim() || "");
const cards = computed(() => (Array.isArray(props.packageEntry?.cards) ? props.packageEntry.cards : []));
const cardCount = computed(() => cards.value.length);
const averageManaValue = computed(() => {
  if (cards.value.length === 0) {
    return "0.00";
  }

  const totalManaValue = cards.value.reduce((sum, card) => sum + (Number(card?.mana_value) || 0), 0);
  return (totalManaValue / cards.value.length).toFixed(2);
});
const colorProfile = computed(() => {
  const colorOrder = ["W", "U", "B", "R", "G"];
  const colors = new Set();

  for (const card of cards.value) {
    const manaCost = typeof card?.mana_cost === "string" ? card.mana_cost : "";
    for (const color of colorOrder) {
      if (manaCost.includes(`{${color}}`)) {
        colors.add(color);
      }
    }
  }

  return colorOrder.filter((color) => colors.has(color)).join("") || "Colorless";
});
const colorProfileSymbols = computed(() => {
  if (colorProfile.value === "Colorless") {
    return "";
  }

  return [...colorProfile.value].map((color) => `{${color}}`).join("");
});

function goToPackageEditor() {
  router.push(`/package-editor/${props.packageEntry.id}`);
}

function openRenameDialog() {
  actionsMenuOpen.value = false;
  pendingName.value = packageName.value;
  renameDialogOpen.value = true;
}

function openDescriptionDialog() {
  actionsMenuOpen.value = false;
  pendingDescription.value = props.packageEntry?.description || "";
  descriptionDialogOpen.value = true;
}

async function submitRename() {
  if (isRenaming.value) {
    return;
  }

  try {
    isRenaming.value = true;
    const renamedPackage = await setPackageNameCommand(props.packageEntry, pendingName.value);
    emit("package-renamed", renamedPackage);
    renameDialogOpen.value = false;
  } catch (e) {
    emit("package-action-error", `Failed to rename package: ${String(e)}`);
    console.error(e);
  } finally {
    isRenaming.value = false;
  }
}

async function submitDescription() {
  if (isSavingDescription.value) {
    return;
  }

  try {
    isSavingDescription.value = true;
    const updatedPackage = await setPackageDescriptionCommand(props.packageEntry.id, pendingDescription.value);
    emit("package-description-updated", updatedPackage);
    descriptionDialogOpen.value = false;
  } catch (e) {
    emit("package-action-error", `Failed to save package description: ${String(e)}`);
    console.error(e);
  } finally {
    isSavingDescription.value = false;
  }
}

async function submitDuplicate() {
  try {
    actionsMenuOpen.value = false;
    const duplicatedPackage = await duplicatePackageCommand(props.packageEntry.id);
    emit("package-duplicated", duplicatedPackage);
  } catch (e) {
    emit("package-action-error", `Failed to duplicate package: ${String(e)}`);
    console.error(e);
  }
}

async function submitDelete() {
  try {
    actionsMenuOpen.value = false;
    await deletePackageCommand(props.packageEntry.id);
    emit("package-deleted", props.packageEntry.id);
  } catch (e) {
    emit("package-action-error", `Failed to delete package: ${String(e)}`);
    console.error(e);
  }
}
</script>

<template>
  <v-container>
    <v-card
      class="package-tile deck-tile"
      width="250"
      height="250"
      max-height="500"
      max-width="500"
      @click="goToPackageEditor"
    >
      <div class="tile-header">
        <h3 class="tile-item">{{ packageName }}</h3>
        <v-menu v-model="actionsMenuOpen" location="bottom">
          <template #activator="{ props: menuProps }">
            <v-btn v-bind="menuProps" class="tile-button" height="25" width="25" size="small" @click.stop>
              <v-icon :icon="mdiDotsHorizontal" size="16"></v-icon>
            </v-btn>
          </template>
          <v-list density="compact">
            <v-list-item title="Rename Package" @click.stop="openRenameDialog">
              <template #prepend>
                <v-icon :icon="mdiPencil"></v-icon>
              </template>
            </v-list-item>
            <v-list-item title="Edit Description" @click.stop="openDescriptionDialog">
              <template #prepend>
                <v-icon :icon="mdiPencil"></v-icon>
              </template>
            </v-list-item>
            <v-list-item title="Duplicate Package" @click.stop="submitDuplicate">
              <template #prepend>
                <v-icon :icon="mdiContentCopy"></v-icon>
              </template>
            </v-list-item>
            <v-list-item title="Delete Package" @click.stop="submitDelete">
              <template #prepend>
                <v-icon :icon="mdiTrashCan"></v-icon>
              </template>
            </v-list-item>
          </v-list>
        </v-menu>
      </div>

      <div class="package-copy">
        {{ packageDescription || "Add a description to explain what this package is for." }}
      </div>

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
      </div>
    </v-card>

    <v-dialog v-model="renameDialogOpen" max-width="420">
      <v-card @click.stop>
        <v-card-title>Rename Package</v-card-title>
        <v-card-text>
          <v-text-field v-model="pendingName" autofocus hide-details label="Package Name" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="renameDialogOpen = false">Cancel</v-btn>
          <v-btn :loading="isRenaming" @click="submitRename">Change Name</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="descriptionDialogOpen" max-width="520">
      <v-card @click.stop>
        <v-card-title>Edit Package Description</v-card-title>
        <v-card-text>
          <v-textarea
            v-model="pendingDescription"
            auto-grow
            counter="220"
            hide-details="auto"
            label="Description"
            rows="4"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="descriptionDialogOpen = false">Cancel</v-btn>
          <v-btn :loading="isSavingDescription" @click="submitDescription">Save Description</v-btn>
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

.package-copy {
  margin-top: 8px;
  color: #5f6f86;
  position: relative;
  z-index: 1;
  display: -webkit-box;
  min-height: 3.9em;
  overflow: hidden;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  line-height: 1.3;
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
