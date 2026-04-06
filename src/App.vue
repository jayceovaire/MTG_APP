<script setup>
import SideNavBar from "./components/SideNavBar.vue";
import { useRoute } from "vue-router";
import { computed, onMounted, ref } from "vue";
import { checkForUpdatesCommand, installUpdateCommand } from "./api/commands.js";

const route = useRoute();
const isCardViewer = computed(() => route.path === "/card-viewer");

const isCheckingForStartupUpdate = ref(false);
const isInstallingUpdate = ref(false);
const showUpdateDialog = ref(false);
const startupUpdate = ref(null);
const startupUpdateError = ref("");

async function checkForStartupUpdate() {
  isCheckingForStartupUpdate.value = true;
  startupUpdateError.value = "";

  try {
    const result = await checkForUpdatesCommand();
    if (result.available) {
      startupUpdate.value = result;
      showUpdateDialog.value = true;
    }
  } catch (error) {
    console.error("Failed to check for updates on startup:", error);
    startupUpdateError.value = String(error);
  } finally {
    isCheckingForStartupUpdate.value = false;
  }
}

function dismissStartupUpdate() {
  showUpdateDialog.value = false;
}

async function installStartupUpdate() {
  if (!startupUpdate.value) {
    return;
  }

  isInstallingUpdate.value = true;
  startupUpdateError.value = "";

  try {
    await installUpdateCommand();
  } catch (error) {
    startupUpdateError.value = `Failed to install update: ${error}`;
    isInstallingUpdate.value = false;
  }
}

onMounted(() => {
  void checkForStartupUpdate();
});
</script>

<template>
  <v-app>
    <SideNavBar v-if="!isCardViewer" />
    <v-main>
      <router-view />
    </v-main>

    <v-dialog v-model="showUpdateDialog" max-width="520">
      <v-card>
        <v-card-title class="text-h6">Update Available</v-card-title>
        <v-card-text class="pt-2">
          <p class="mb-3">
            Version
            <strong>{{ startupUpdate?.version }}</strong>
            is available. You are currently running
            <strong>{{ startupUpdate?.currentVersion }}</strong>.
          </p>

          <p v-if="startupUpdate?.body" class="text-body-2 text-medium-emphasis mb-0">
            {{ startupUpdate.body }}
          </p>

          <p v-if="startupUpdateError" class="text-body-2 text-error mt-3 mb-0">
            {{ startupUpdateError }}
          </p>
        </v-card-text>
        <v-card-actions class="px-6 pb-4">
          <v-spacer />
          <v-btn
            variant="text"
            :disabled="isInstallingUpdate || isCheckingForStartupUpdate"
            @click="dismissStartupUpdate"
          >
            No
          </v-btn>
          <v-btn
            color="primary"
            variant="tonal"
            :loading="isInstallingUpdate"
            :disabled="isCheckingForStartupUpdate"
            @click="installStartupUpdate"
          >
            Update
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-app>
</template>

<style scoped>

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color-scheme: dark;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  padding: 0;
  background-color: #0f0f0f;
}
</style>
