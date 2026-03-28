<script setup>
import { mdiPackageVariantClosed, mdiPlus } from "@mdi/js";
import { onMounted, ref } from "vue";
import PackageTile from "../components/PackageTile.vue";
import { createPackageCommand, getPackagesCommand } from "../api/deckCommands.js";

const packages = ref([]);
const isCreatingPackage = ref(false);
const snackbarVisible = ref(false);
const snackbarMessage = ref("");
const snackbarColor = ref("success");

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
  packages.value = await getPackagesCommand();
});

function handlePackageRenamed(updatedPackage) {
  const packageIndex = packages.value.findIndex((entry) => entry.id === updatedPackage.id);
  if (packageIndex === -1) {
    return;
  }

  packages.value[packageIndex] = updatedPackage;
  showSuccess(`Renamed package to "${updatedPackage.name}"`);
}

function handlePackageDescriptionUpdated(updatedPackage) {
  const packageIndex = packages.value.findIndex((entry) => entry.id === updatedPackage.id);
  if (packageIndex === -1) {
    return;
  }

  packages.value[packageIndex] = updatedPackage;
  showSuccess(`Updated description for "${updatedPackage.name}"`);
}

async function handleCreatePackage() {
  if (isCreatingPackage.value) {
    return;
  }

  try {
    isCreatingPackage.value = true;
    const newPackage = await createPackageCommand("Untitled Package");
    packages.value.push(newPackage);
    showSuccess(`Created package "${newPackage.name}"`);
  } catch (e) {
    showError(`Failed to create package: ${String(e)}`);
    console.error(e);
  } finally {
    isCreatingPackage.value = false;
  }
}

function handlePackageDeleted(deletedId) {
  packages.value = packages.value.filter((entry) => entry.id !== deletedId);
  showSuccess("Package deleted");
}

function handlePackageDuplicated(duplicatedPackage) {
  packages.value.push(duplicatedPackage);
  showSuccess(`Duplicated package as "${duplicatedPackage.name}"`);
}

function handlePackageActionError(message) {
  showError(message);
}
</script>

<template>
  <v-container>
    <div class="deck-library-header">
      <v-btn class="create-deck-btn" :loading="isCreatingPackage" @click="handleCreatePackage">
        <template #prepend>
          <span class="create-deck-icons" aria-hidden="true">
            <v-icon :icon="mdiPlus" size="16"></v-icon>
            <v-icon :icon="mdiPackageVariantClosed" size="16"></v-icon>
          </span>
        </template>
        Create Package
      </v-btn>
    </div>

    <div class="deck-grid">
      <PackageTile
        v-for="packageEntry in packages"
        :key="packageEntry.id"
        :package-entry="packageEntry"
        @package-renamed="handlePackageRenamed"
        @package-description-updated="handlePackageDescriptionUpdated"
        @package-deleted="handlePackageDeleted"
        @package-duplicated="handlePackageDuplicated"
        @package-action-error="handlePackageActionError"
      />
    </div>

    <v-snackbar v-model="snackbarVisible" :color="snackbarColor" :timeout="2500">
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
