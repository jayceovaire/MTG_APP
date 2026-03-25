<script setup>
import { mdiDotsHorizontal, mdiHeart, mdiHeartOutline, mdiPlus, mdiTrashCan } from "@mdi/js";
import { computed, ref } from "vue";
import ManaText from "./ManaText.vue";

const props = defineProps({
  name: {
    type: String,
    default: "Unnamed Card",
  },
  manaCost: {
    type: String,
    default: "",
  },
  typeLine: {
    type: String,
    default: "Card",
  },
  oracleText: {
    type: String,
    default: "",
  },
  power: {
    type: [String, Number],
    default: null,
  },
  toughness: {
    type: [String, Number],
    default: null,
  },
  image: {
    type: String,
    default: "",
  },
  favorited: {
    type: Boolean,
    default: false,
  },
});

const actionsMenuOpen = ref(false);
const emit = defineEmits(["remove-card", "add-card", "favorite-card"]);
const displayName = computed(() => props.name.split(" // ")[0]?.trim() || props.name);

function handleRemoveCard() {
  actionsMenuOpen.value = false;
  emit("remove-card");
}

function handleAddCard() {
  actionsMenuOpen.value = false;
  emit("add-card");
}

function handleFavoriteCard() {
  actionsMenuOpen.value = false;
  emit("favorite-card");
}
</script>

<template>
  <article class="mtg-card">
    <div class="card-actions">
      <v-menu v-model="actionsMenuOpen" location="bottom">
        <template #activator="{ props: menuProps }">
          <v-btn v-bind="menuProps" @click.stop size="small" class="tile-button" height="16" width="25">
            <v-icon :icon="mdiDotsHorizontal" size="16"></v-icon>
          </v-btn>
        </template>
        <v-list density="compact">
          <v-list-item @click.stop="handleAddCard" title="Add Card">
            <template #prepend>
              <v-icon :icon="mdiPlus"></v-icon>
            </template>
          </v-list-item>
          <v-list-item @click.stop="handleFavoriteCard" :title="favorited ? 'Unfavorite Card' : 'Favorite Card'">
            <template #prepend>
              <v-icon :icon="favorited ? mdiHeart : mdiHeartOutline"></v-icon>
            </template>
          </v-list-item>
          <v-list-item @click.stop="handleRemoveCard" title="Remove Card">
            <template #prepend>
              <v-icon :icon="mdiTrashCan"></v-icon>
            </template>
          </v-list-item>
        </v-list>
      </v-menu>
    </div>

    <header class="card-row card-title-row">
      <span class="card-name">{{ displayName }}</span>
      <ManaText class="card-mana" :text="manaCost" empty-text="-" :cost="true" />
    </header>

    <section class="card-art">
      <img v-if="image" :src="image" :alt="displayName" />
      <div v-else class="card-art-placeholder">Art</div>
    </section>

    <section class="card-row card-type-row">
      <span>{{ typeLine }}</span>
    </section>

    <section class="card-text-box">
      <p>
        <ManaText :text="oracleText" empty-text="No rules text." :multiline="true" />
      </p>
    </section>

    <footer class="card-footer">
      <span class="rarity-dot" aria-hidden="true"></span>
      <span class="pt-badge" v-if="power !== null && toughness !== null">
        {{ power }}/{{ toughness }}
      </span>
    </footer>
  </article>
</template>

<style scoped>
.mtg-card {
  color: #000;
  width: 250px;
  min-height: 360px;
  padding: 8px;
  border-radius: 14px;
  border: 2px solid #5e5241;
  background: linear-gradient(165deg, #d9caa5 0%, #c7b084 50%, #b6976f 100%);
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.22);
  display: grid;
  grid-template-rows: auto auto 105px auto 1fr auto;
  gap: 6px;
}

.card-actions {
  display: flex;
  justify-content: flex-end;
}

.tile-button {
  min-width: 24px !important;
  width: 24px;
  min-height: 16px !important;
  height: 16px;
  padding: 0;
}

.card-row {
  border: 1px solid #6f6048;
  border-radius: 8px;
  background: #efe6d1;
  padding: 4px 6px;
}

.card-title-row {
  display: flex;
  gap: 6px;
  align-items: center;
  font-weight: 700;
}

.card-name {
  flex: 1 1 auto;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-mana {
  flex: 0 0 auto;
  font-weight: 600;
}

.card-art {
  border: 1px solid #6f6048;
  border-radius: 8px;
  background: #d8d0bb;
  overflow: hidden;
}

.card-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.card-art-placeholder {
  height: 100%;
  display: grid;
  place-items: center;
  color: #5e5241;
  font-weight: 600;
}

.card-type-row {
  font-size: 0.9rem;
  font-weight: 600;
}

.card-text-box {
  border: 1px solid #6f6048;
  border-radius: 8px;
  background: #efe6d1;
  padding: 8px;
  font-size: 0.86rem;
  line-height: 1.3;
}

.card-text-box p {
  margin: 0;
  white-space: pre-line;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.rarity-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #b28a2f;
  border: 1px solid #6f6048;
}

.pt-badge {
  border: 1px solid #6f6048;
  border-radius: 10px;
  background: #efe6d1;
  padding: 2px 8px;
  font-weight: 700;
}
</style>
