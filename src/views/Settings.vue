<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { 
  checkForUpdatesCommand, 
  fetchCardImagesCommand 
} from '../api/commands';
import { mdiUpdate, mdiDownload, mdiCached, mdiInformationOutline, mdiPaletteOutline } from '@mdi/js';

const appVersion = ref('0.1.0');
const isCheckingUpdate = ref(false);
const updateMessage = ref('');
const updateColor = ref('info');

const isDownloading = ref(false);
const downloadProgress = ref(0);
const currentCardName = ref('');
const downloadTotal = ref(0);
const downloadCurrent = ref(0);

let unlistenProgress = null;
let unlistenComplete = null;

async function handleCheckUpdate() {
  isCheckingUpdate.value = true;
  updateMessage.value = 'Checking for updates...';
  updateColor.value = 'info';
  try {
    const result = await checkForUpdatesCommand();
    updateMessage.value = result;
    updateColor.value = 'success';
  } catch (error) {
    updateMessage.value = `Update check failed: ${error}`;
    updateColor.value = 'error';
  } finally {
    isCheckingUpdate.value = false;
  }
}

async function handleDownloadAllImages() {
  isDownloading.value = true;
  downloadProgress.value = 0;
  currentCardName.value = 'Initializing...';
  downloadTotal.value = 0;
  downloadCurrent.value = 0;

  try {
    // Call the command with all=true
    await fetchCardImagesCommand(null, null, null, true);
  } catch (error) {
    console.error('Failed to start image download:', error);
    isDownloading.value = false;
  }
}

onMounted(async () => {
  unlistenProgress = await listen('image-download-progress', (event) => {
    const payload = event.payload;
    downloadCurrent.value = payload.current;
    downloadTotal.value = payload.total;
    currentCardName.value = payload.card_name;
    if (payload.total > 0) {
      downloadProgress.value = Math.round((payload.current / payload.total) * 100);
    }
  });

  unlistenComplete = await listen('image-download-complete', () => {
    isDownloading.value = false;
  });
});

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
  if (unlistenComplete) unlistenComplete();
});
</script>

<template>
  <v-container class="settings-page">
    <div class="d-flex align-center mb-6">
      <h1 class="text-h4 font-weight-bold">Settings</h1>
    </div>

    <v-row>
      <!-- App Information -->
      <v-col cols="12" md="6">
        <v-card variant="flat" border class="pa-4 mb-6">
          <div class="d-flex align-center mb-4">
            <v-icon :icon="mdiInformationOutline" color="primary" class="mr-2"></v-icon>
            <h2 class="text-h6">Application Information</h2>
          </div>
          
          <v-list density="compact" class="bg-transparent pa-0">
            <v-list-item class="px-0">
              <template v-slot:prepend>
                <div class="text-subtitle-2 text-medium-emphasis mr-4" style="width: 120px">Version</div>
              </template>
              <v-list-item-title class="text-body-2 font-weight-bold">{{ appVersion }}</v-list-item-title>
            </v-list-item>
            
            <v-list-item class="px-0">
              <template v-slot:prepend>
                <div class="text-subtitle-2 text-medium-emphasis mr-4" style="width: 120px">Update</div>
              </template>
              <div class="d-flex align-center">
                <v-btn
                  size="small"
                  variant="tonal"
                  color="primary"
                  :loading="isCheckingUpdate"
                  :prepend-icon="mdiUpdate"
                  @click="handleCheckUpdate"
                >
                  Check for Updates
                </v-btn>
                <span v-if="updateMessage" :class="`text-caption ml-3 text-${updateColor}`">
                  {{ updateMessage }}
                </span>
              </div>
            </v-list-item>
          </v-list>
        </v-card>

        <!-- Appearance (Placeholder) -->
        <v-card variant="flat" border class="pa-4">
          <div class="d-flex align-center mb-4">
            <v-icon :icon="mdiPaletteOutline" color="primary" class="mr-2"></v-icon>
            <h2 class="text-h6">Appearance</h2>
          </div>
          
          <v-list density="compact" class="bg-transparent pa-0">
            <v-list-item class="px-0">
              <v-switch
                label="Dark Mode"
                model-value="true"
                color="primary"
                hide-details
                disabled
              ></v-switch>
              <div class="text-caption text-medium-emphasis mt-n2 ml-1">
                System is currently locked to dark mode.
              </div>
            </v-list-item>
          </v-list>
        </v-card>
      </v-col>

      <!-- Cache & Data Management -->
      <v-col cols="12" md="6">
        <v-card variant="flat" border class="pa-4">
          <div class="d-flex align-center mb-4">
            <v-icon :icon="mdiCached" color="primary" class="mr-2"></v-icon>
            <h2 class="text-h6">Cache & Data</h2>
          </div>

          <p class="text-body-2 text-medium-emphasis mb-4">
            Manage your local card image cache. Downloading all images may take some time depending on your collection size.
          </p>

          <v-divider class="mb-4"></v-divider>

          <div class="d-flex flex-column gap-2">
            <v-btn
              block
              variant="tonal"
              color="primary"
              :prepend-icon="mdiDownload"
              @click="handleDownloadAllImages"
              :disabled="isDownloading"
            >
              Download All Scryfall Images
            </v-btn>
            
            <div class="text-caption text-medium-emphasis mt-2">
              <v-icon size="x-small" class="mr-1">mdi-information</v-icon>
              Uses global rate limiting (10 cards/sec) to comply with Scryfall's API policy.
            </div>
          </div>
        </v-card>
      </v-col>
    </v-row>

    <!-- Download Progress Modal -->
    <v-dialog v-model="isDownloading" persistent max-width="500">
      <v-card class="pa-4">
        <v-card-title class="text-h6 d-flex align-center">
          <v-progress-circular
            indeterminate
            size="24"
            width="3"
            color="primary"
            class="mr-3"
          ></v-progress-circular>
          Downloading Images
        </v-card-title>
        
        <v-card-text class="pt-4">
          <div class="d-flex justify-space-between align-center mb-2">
            <span class="text-body-2 text-truncate" style="max-width: 300px">
              {{ currentCardName }}
            </span>
            <span class="text-caption text-medium-emphasis">
              {{ downloadCurrent }} / {{ downloadTotal }}
            </span>
          </div>
          
          <v-progress-linear
            v-model="downloadProgress"
            color="primary"
            height="10"
            rounded
            striped
          ></v-progress-linear>
          
          <div class="text-center mt-4 text-body-2">
            {{ downloadProgress }}% Complete
          </div>
          
          <div class="text-caption text-center text-medium-emphasis mt-2">
            Please do not close the application or navigate away.
          </div>
        </v-card-text>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<style scoped>
.settings-page {
  max-width: 1000px;
  margin: 0 auto;
}

.gap-2 {
  gap: 8px;
}
</style>