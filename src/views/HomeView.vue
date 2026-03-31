<script setup>
import { computed, onMounted, ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { 
  getMostRecentCachedImageCommand, 
  runTestCommand, 
  fetchCardImagesCommand,
  getRandomCardCommand,
  createCollectionCardCommand
} from "../api/commands.js";

const testGreetingInput = ref("string from HomeView.vue");
const testCommandResult = ref("");
const testCommandError = ref("");
const recentImage = ref("");
const imageError = ref("");
const isFetching = ref(false);

const randomCard = ref(null);
const isFetchingRandom = ref(false);
const randomCardError = ref("");

const randomCardImage = computed(() => {
  if (!randomCard.value || !randomCard.value.image) return "";
  const image = randomCard.value.image;
  if (image.startsWith("http") || image.startsWith("data:")) {
    return image;
  }
  return convertFileSrc(image);
});

async function handleGetRandomCard() {
  isFetchingRandom.value = true;
  randomCard.value = null;
  randomCardError.value = "";
  try {
    randomCard.value = await getRandomCardCommand();
  } catch (error) {
    randomCardError.value = String(error);
  } finally {
    isFetchingRandom.value = false;
  }
}

async function handleAddToCollection() {
  if (!randomCard.value) return;
  try {
    await createCollectionCardCommand(randomCard.value.name);
    // You might want to show a success message here
  } catch (error) {
    console.error("Failed to add to collection:", error);
  }
}

async function handleTestCommand() {
  testCommandError.value = "";

  try {
    testCommandResult.value = await runTestCommand(testGreetingInput.value);
  } catch (error) {
    testCommandResult.value = "";
    testCommandError.value = String(error);
  }
}

async function loadRecentImage() {
  try {
    recentImage.value = await getMostRecentCachedImageCommand();
  } catch (error) {
    imageError.value = String(error);
  }
}

async function handleFetchImages() {
  isFetching.value = true;
  imageError.value = "";
  try {
    await fetchCardImagesCommand();
    await loadRecentImage();
  } catch (error) {
    imageError.value = String(error);
  } finally {
    isFetching.value = false;
  }
}

  onMounted(async () => {
    await handleFetchImages();
    await handleGetRandomCard();
  });
</script>

<template>
  <v-container class="home-page">
    <h1>Home</h1>
    <p>This page includes a simple frontend-to-Rust command example.</p>

    <div class="home-grid">
      <section class="command-reference random-card-section">
        <h2>Random Card Discovery</h2>
        
        <div class="random-card-container">
          <div class="card-display">
            <div v-if="isFetchingRandom" class="d-flex align-center justify-center" style="height: 100%">
              <v-progress-circular indeterminate></v-progress-circular>
            </div>
            <template v-else-if="randomCard">
              <img v-if="randomCardImage" :src="randomCardImage" :alt="randomCard.name" class="random-card-img" />
              <div v-else class="card-art-placeholder">No Image Available</div>
            </template>
            <div v-else class="card-art-placeholder">No Card Found</div>
          </div>
          
          <v-btn 
            class="mt-2" 
            @click="handleAddToCollection" 
            color="success" 
            block 
            :disabled="isFetchingRandom || !randomCard"
          >
            Add to Collection
          </v-btn>
        </div>
        
        <div v-if="randomCardError" class="command-error">
          <strong>Error:</strong>
          <span>{{ randomCardError }}</span>
        </div>

        <v-btn class="mt-2" @click="handleGetRandomCard" :disabled="isFetchingRandom" color="primary">
          Discover New Card
        </v-btn>
      </section>

      <section class="command-reference">
        <h2>Recent Cached Image</h2>
        <p v-if="!recentImage && !imageError && !isFetching">No images found in cache.</p>
        <div v-if="isFetching" class="d-flex align-center gap-2">
          <v-progress-circular indeterminate size="24" width="2"></v-progress-circular>
          <span>Fetching images...</span>
        </div>
        <div v-if="recentImage" class="image-container">
          <img :src="recentImage" alt="Recent Cached Card" class="recent-card-img" />
        </div>
        <div v-if="imageError" class="command-error">
          <strong>Image load error:</strong>
          <span>{{ imageError }}</span>
        </div>
        <v-btn class="mt-2" @click="handleFetchImages" size="small" :disabled="isFetching">
          Refresh / Fetch Missing Images
        </v-btn>
      </section>

      <section class="command-reference test-command-section">
        <h2>Test Command Reference</h2>
        <p>
          Listener in this view: `handleTestCommand()`.
          Wrapper in `src/api/commands.js`: `runTestCommand()`.
          Backend Rust command: `test_command`.
        </p>

        <v-text-field
          v-model="testGreetingInput"
          label="Greeting payload"
          hide-details
        />

        <v-btn class="mt-4" @click="handleTestCommand">Run Test Command</v-btn>

        <div class="command-result">
          <strong>Command result:</strong>
          <span>{{ testCommandResult || "-" }}</span>
        </div>

        <div v-if="testCommandError" class="command-error">
          <strong>Command error:</strong>
          <span>{{ testCommandError }}</span>
        </div>
      </section>
    </div>
  </v-container>
</template>

<style scoped>
.home-page {
  color: #132032;
}

.home-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 24px;
  margin-top: 24px;
}

.home-page h1,
.command-reference h2,
.command-reference p {
  margin: 0;
}

.command-reference {
  max-width: 560px;
  display: grid;
  gap: 12px;
  padding: 20px;
  border-radius: 20px;
  background:
    radial-gradient(circle at top left, rgba(187, 214, 255, 0.4), transparent 34%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.96) 0%, rgba(247, 250, 255, 0.98) 100%);
  border: 1px solid rgba(27, 42, 63, 0.08);
  box-shadow: 0 20px 40px rgba(20, 31, 48, 0.05);
  color: #132032;
}

.command-result {
  display: grid;
  gap: 6px;
  color: #132032;
}

.command-error {
  display: grid;
  gap: 6px;
  color: #8b2d27;
}

.random-card-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.card-display {
  width: 100%;
  max-width: 280px;
  aspect-ratio: 2.5 / 3.5;
  border-radius: 14px;
  overflow: hidden;
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.3);
  background: #2a2a2a;
}

.random-card-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
}

.card-art-placeholder {
  height: 100%;
  display: grid;
  place-items: center;
  color: #ccc;
  font-weight: 600;
}

.image-container {
  display: flex;
  justify-content: center;
  padding: 10px;
}

.recent-card-img {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.2);
}
</style>
