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
    <div class="d-flex align-center justify-space-between mb-6">
      <h1 class="text-h4 font-weight-bold">Packages</h1>
      <v-btn
        color="primary"
        :loading="isCreatingPackage"
        @click="handleCreatePackage"
        class="text-none font-weight-bold"
        rounded="lg"
      >
        <template #prepend>
          <div class="d-flex align-center mr-1">
            <v-icon :icon="mdiPlus" size="18"></v-icon>
            <v-icon :icon="mdiPackageVariantClosed" size="18" class="ml-n1"></v-icon>
          </div>
        </template>
        Create Package
      </v-btn>
    </div>

    <v-divider class="mb-6"></v-divider>

    <div v-if="packages.length === 0" class="d-flex flex-column align-center justify-center pa-12 text-medium-emphasis">
      <v-icon :icon="mdiPackageVariantClosed" size="64" class="mb-4 opacity-20"></v-icon>
      <div class="text-h6">No packages found</div>
      <p>Create a new package to reuse groups of cards across decks.</p>
    </div>

    <div v-else class="deck-grid">
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
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.opacity-20 {
  opacity: 0.2;
}
</style>
