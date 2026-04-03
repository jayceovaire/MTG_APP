<script setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { 
  getMostRecentCachedImageCommand, 
  runTestCommand, 
  fetchCardImagesCommand,
  getRandomCardCommand,
  createCollectionCardCommand
} from "../api/commands.js";


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
    await fetchCardImagesCommand(null, null, null, true);
    await loadRecentImage();
  } catch (error) {
    imageError.value = String(error);
  } finally {
    isFetching.value = false;
  }
}

  let unlistenImages = null;

  onMounted(async () => {
    await loadRecentImage();
    await handleGetRandomCard();
    unlistenImages = await listen("images-updated", () => {
      console.log("Images updated event received in Home, reloading recent image...");
      loadRecentImage();
    });
  });

  onUnmounted(() => {
    if (unlistenImages) {
      unlistenImages();
    }
  });
</script>

<template>
  <v-container class="home-page">
    <h1 class="text-primary">Home</h1>

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

    </div>
  </v-container>
</template>

<style scoped>
.home-page {
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
  padding: 24px;
  border-radius: 24px;
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
}

.command-result {
  display: grid;
  gap: 6px;
  opacity: 0.9;
}

.command-error {
  display: grid;
  gap: 6px;
  color: rgb(var(--v-theme-error));
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
