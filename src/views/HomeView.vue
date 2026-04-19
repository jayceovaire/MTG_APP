<script setup>
import { computed, onMounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { releaseNotes } from "../data/releaseNotes.js";
import RandomCardViewer from "../components/RandomCardViewer.vue";


const activeTab = ref("random-card");
const appVersion = ref("0.1.5");

const currentRelease = computed(() => {
  return releaseNotes.find((entry) => entry.version === appVersion.value) ?? releaseNotes[0] ?? null;
});

onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    console.error("Failed to load app version:", error);
  }
});
</script>

<template>
  <v-container class="home-page">
    <div class="home-header">
      <div>
        <h1 class="text-primary">Home</h1>
      </div>
      <div class="version-chip">Version {{ appVersion }}</div>
    </div>

    <v-card class="home-shell" variant="flat">
      <v-tabs
        v-model="activeTab"
        align-tabs="start"
        color="primary"
        bg-color="transparent"
        slider-color="primary"
      >
        <v-tab value="random-card">Random Card Viewer</v-tab>
        <v-tab value="release-notes">Release Notes</v-tab>
      </v-tabs>

      <v-window v-model="activeTab" class="home-window">
        <v-window-item value="random-card">
          <section class="panel-card random-card-section">
            <div v-if="activeTab === 'random-card'" class="mt-4">
              <RandomCardViewer />
            </div>
          </section>
        </v-window-item>

        <v-window-item value="release-notes">
          <section class="panel-card release-notes-section">
            <div class="panel-copy">
              <h2>Release Notes</h2>
              <p v-if="currentRelease" class="text-medium-emphasis mb-0">
                {{ currentRelease.headline }}
              </p>
            </div>

            <template v-if="currentRelease">
              <div class="release-meta">
                <div>
                  <span class="meta-label">Version</span>
                  <strong>{{ currentRelease.version }}</strong>
                </div>
                <div>
                  <span class="meta-label">Published</span>
                  <strong>{{ currentRelease.publishedOn }}</strong>
                </div>
              </div>

              <ul class="release-bullets">
                <li v-for="bullet in currentRelease.bullets" :key="bullet">
                  {{ bullet }}
                </li>
              </ul>

            </template>

            <p v-else class="text-medium-emphasis mb-0">
              No release notes are available for this version yet.
            </p>
          </section>
        </v-window-item>
      </v-window>
    </v-card>
  </v-container>
</template>

<style scoped>
.home-page {
  display: grid;
  gap: 24px;
}

.home-header {
  display: flex;
  justify-content: space-between;
  align-items: end;
  gap: 16px;
  flex-wrap: wrap;
}

.home-header h1,
.panel-card h2,
.panel-card h3,
.panel-card p {
  margin: 0;
}

.version-chip {
  padding: 10px 14px;
  border-radius: 999px;
  border: 1px solid rgba(33, 150, 243, 0.35);
  background: rgba(33, 150, 243, 0.12);
  color: rgb(var(--v-theme-primary));
  font-weight: 700;
}

.home-shell {
  padding: 12px;
  border-radius: 28px;
  background:
    radial-gradient(circle at top left, rgba(33, 150, 243, 0.16), transparent 28%),
    linear-gradient(180deg, rgba(var(--v-border-color), 0.02), rgba(var(--v-border-color), 0.01)),
    rgb(var(--v-theme-surface));
  border: 1px solid rgba(var(--v-border-color), 0.08);
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.28);
}

.home-window {
  margin-top: 12px;
}

.panel-card {
  display: grid;
  gap: 20px;
  padding: 24px;
  border-radius: 22px;
  background: rgba(var(--v-border-color), 0.02);
  border: 1px solid rgba(var(--v-border-color), 0.05);
}

.panel-copy {
  display: grid;
  gap: 8px;
}

.release-meta {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.release-meta > div {
  display: grid;
  gap: 4px;
}

.meta-label {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: rgba(var(--v-theme-on-surface), 0.6);
}

.release-bullets {
  display: grid;
  gap: 12px;
  padding-left: 20px;
  margin: 0;
}

.release-bullets li {
  line-height: 1.6;
}

.commit-block {
  display: grid;
  gap: 12px;
}

.commit-list {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.commit-chip {
  padding: 8px 12px;
  border-radius: 999px;
  background: rgba(var(--v-border-color), 0.06);
  border: 1px solid rgba(var(--v-border-color), 0.08);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 0.9rem;
}
</style>
