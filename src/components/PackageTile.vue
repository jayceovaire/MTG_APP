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
    class="package-tile deck-tile pa-4 h-100 deck-card-hover"
    variant="flat"
    border
    @click="goToPackageEditor"
  >
    <div class="d-flex align-center gap-4 mb-3">
      <v-icon :icon="mdiPackageVariantClosed" size="32" color="primary"></v-icon>
      <div class="overflow-hidden">
        <h3 class="text-h6 font-weight-bold text-truncate">{{ packageName }}</h3>
        <div class="text-caption text-medium-emphasis">{{ cardCount }} Cards</div>
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
          <v-list-item @click.stop="openDescriptionDialog">
            <template #prepend>
              <v-icon :icon="mdiPencil" size="18"></v-icon>
            </template>
            <v-list-item-title>Edit Description</v-list-item-title>
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

    <div class="text-body-2 text-medium-emphasis mb-4 package-description">
      {{ packageDescription || "Add a description to explain what this package is for." }}
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
.package-tile {
  cursor: pointer;
  transition: transform 0.2s, border-color 0.2s;
}

.package-tile:hover {
  transform: translateY(-4px);
}

.deck-card-hover:hover {
  border-color: rgb(var(--v-theme-primary)) !important;
  background-color: rgba(var(--v-theme-primary), 0.02) !important;
}

.gap-4 {
  gap: 16px;
}

.package-description {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  height: 4.5em;
  line-height: 1.5;
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
