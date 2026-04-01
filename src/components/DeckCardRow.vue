<script setup>
import {
  mdiAccountMultiplePlusOutline,
  mdiCancel,
  mdiCloseCircleOutline,
  mdiCrownOutline,
  mdiDotsHorizontal,
  mdiHeart,
  mdiHeartOutline,
  mdiPackageVariantClosedPlus,
  mdiGaugeFull,
} from "@mdi/js";
import { computed } from "vue";
import ManaText from "./ManaText.vue";

const props = defineProps({
  card: {
    type: Object,
    required: true,
  },
  quantity: {
    type: Number,
    default: 1,
  },
  editable: {
    type: Boolean,
    default: true,
  },
  canSetCommander: {
    type: Boolean,
    default: false,
  },
  canSetPartner: {
    type: Boolean,
    default: false,
  },
  canRemovePartner: {
    type: Boolean,
    default: false,
  },
  canRemoveCommander: {
    type: Boolean,
    default: false,
  },
  favorited: {
    type: Boolean,
    default: false,
  },
  showFavoriteIndicator: {
    type: Boolean,
    default: false,
  },
  showFavoriteAction: {
    type: Boolean,
    default: false,
  },
  showAddToPackageAction: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits([
  "add-copy",
  "remove-copy",
  "set-commander",
  "set-partner",
  "remove-commander",
  "remove-partner",
  "favorite-card",
  "add-to-package",
]);
const hasContextActions = computed(() => (
  props.canSetCommander ||
  props.canSetPartner ||
  props.canRemovePartner ||
  props.canRemoveCommander ||
  props.showFavoriteAction ||
  props.showAddToPackageAction
));

function buildTypeLine(card) {
  const superType = Array.isArray(card.super_type) ? card.super_type.join(" ") : "";
  const cardType = Array.isArray(card.card_type) ? card.card_type.join(" ") : "Card";
  const subType = Array.isArray(card.sub_type) && card.sub_type.length > 0 ? ` - ${card.sub_type.join(" ")}` : "";
  return `${superType} ${cardType}${subType}`.trim();
}

function commanderLegality(card) {
  if (!card || typeof card !== "object") {
    return "";
  }
  const v = card.commander_legality ?? card.commanderLegality;
  return typeof v === "string" ? v : "";
}

function isGameChangerCard(card) {
  if (!card || typeof card !== "object") {
    return false;
  }
  return card.game_changer === true || card.gameChanger === true;
}

/** Banned, not legal in Commander, restricted, etc. (anything other than `legal`). */
const illegalInCommander = computed(() => {
  const leg = commanderLegality(props.card).trim().toLowerCase();
  if (!leg) {
    return false;
  }
  return leg !== "legal";
});

const isGameChanger = computed(() => isGameChangerCard(props.card));
</script>

<template>
  <article
    class="deck-card-row"
    :class="{
      'deck-card-row--illegal': illegalInCommander,
      'deck-card-row--game-changer': isGameChanger && !illegalInCommander,
    }"
  >
    <div
      class="deck-card-row__count"
      :class="{
        'deck-card-row__count--illegal': illegalInCommander,
        'deck-card-row__count--game-changer': isGameChanger && !illegalInCommander,
      }"
    >
      {{ quantity }}x
      <v-icon
        v-if="isGameChanger && !illegalInCommander"
        :icon="mdiGaugeFull"
        size="12"
        class="ml-1"
      ></v-icon>
      <v-icon
        v-if="illegalInCommander"
        :icon="mdiCancel"
        size="12"
        class="ml-1"
      ></v-icon>
    </div>
    <div class="deck-card-row__body">
      <div class="deck-card-row__top">
        <span class="deck-card-row__name">
          <span>{{ card.name }}</span>
          <span
            v-if="illegalInCommander"
            class="deck-card-row__status-pill deck-card-row__status-pill--illegal"
          >
            {{
              commanderLegality(card).toUpperCase().replace("_", " ") || "NOT LEGAL"
            }}
          </span>
          <span
            v-if="isGameChanger"
            class="deck-card-row__status-pill deck-card-row__status-pill--game-changer"
          >
            GAME CHANGER
          </span>
          <v-tooltip v-if="illegalInCommander" location="top">
            <template #activator="{ props: tipProps }">
              <v-icon
                v-bind="tipProps"
                :icon="mdiCancel"
                size="18"
                class="deck-card-row__illegal"
                aria-label="Not legal in Commander"
              ></v-icon>
            </template>
            <span>Not legal in Commander (banned or not legal in format)</span>
          </v-tooltip>
          <v-tooltip v-if="isGameChanger" location="top">
            <template #activator="{ props: tipProps }">
              <v-icon
                v-bind="tipProps"
                :icon="mdiGaugeFull"
                size="18"
                class="deck-card-row__game-changer"
                aria-label="Game changer"
              ></v-icon>
            </template>
            <span>Game changer (Commander)</span>
          </v-tooltip>
          <v-icon
            v-if="showFavoriteIndicator && favorited"
            :icon="mdiHeart"
            size="16"
            class="deck-card-row__favorite"
          ></v-icon>
        </span>
        <div class="deck-card-row__actions">
          <ManaText class="deck-card-row__mana" :text="card.mana_cost || ''" empty-text="-" :cost="true" />
          <div v-if="editable" class="deck-card-row__buttons">
            <v-btn size="x-small" variant="outlined" icon @click.stop="$emit('remove-copy')">
              <span class="deck-card-row__button-glyph">-</span>
            </v-btn>
            <v-btn size="x-small" variant="outlined" icon @click.stop="$emit('add-copy')">
              <span class="deck-card-row__button-glyph">+</span>
            </v-btn>
            <v-menu v-if="hasContextActions" location="bottom end">
              <template #activator="{ props: menuProps }">
                <v-btn v-bind="menuProps" size="x-small" variant="outlined" icon @click.stop>
                  <v-icon :icon="mdiDotsHorizontal" size="14"></v-icon>
                </v-btn>
              </template>
              <v-list density="compact">
                <v-list-item
                  v-if="canSetCommander"
                  title="Set as Commander"
                  @click.stop="$emit('set-commander')"
                >
                  <template #prepend>
                    <v-icon :icon="mdiCrownOutline" size="16"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  v-if="canSetPartner"
                  title="Set as Partner"
                  @click.stop="$emit('set-partner')"
                >
                  <template #prepend>
                    <v-icon :icon="mdiAccountMultiplePlusOutline" size="16"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  v-if="canRemovePartner"
                  title="Remove as Partner"
                  @click.stop="$emit('remove-partner')"
                  class="text-error"
                >
                  <template #prepend>
                    <v-icon :icon="mdiCloseCircleOutline" size="16" color="error"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  v-if="canRemoveCommander"
                  title="Remove as Commander"
                  @click.stop="$emit('remove-commander')"
                  class="text-error"
                >
                  <template #prepend>
                    <v-icon :icon="mdiCloseCircleOutline" size="16" color="error"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  v-if="showFavoriteAction"
                  :title="favorited ? 'Unfavorite Card' : 'Favorite Card'"
                  @click.stop="$emit('favorite-card')"
                >
                  <template #prepend>
                    <v-icon :icon="favorited ? mdiHeart : mdiHeartOutline" size="16"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  v-if="showAddToPackageAction"
                  title="Add to Package"
                  @click.stop="$emit('add-to-package')"
                >
                  <template #prepend>
                    <v-icon :icon="mdiPackageVariantClosedPlus" size="16"></v-icon>
                  </template>
                </v-list-item>
              </v-list>
            </v-menu>
          </div>
        </div>
      </div>
      <div class="deck-card-row__bottom">
        <span>{{ buildTypeLine(card) }}</span>
        <span>MV {{ card.mana_value ?? 0 }}</span>
      </div>
    </div>
  </article>
</template>

<style scoped>
.deck-card-row {
  display: grid;
  grid-template-columns: 52px minmax(0, 1fr);
  gap: 12px;
  align-items: start;
  padding: 10px 12px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
  container-type: inline-size;
}

.deck-card-row--illegal {
  border-color: rgba(var(--v-theme-error), 0.5);
  background: rgba(var(--v-theme-error), 0.05);
}

.deck-card-row--game-changer {
  border-color: rgba(255, 193, 7, 0.3);
  background: rgba(255, 193, 7, 0.05);
}

.deck-card-row__illegal {
  color: rgb(var(--v-theme-error));
  flex: 0 0 auto;
}

.deck-card-row__game-changer {
  color: #ffc107;
  flex: 0 0 auto;
}

.deck-card-row__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 36px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.1);
  color: inherit;
  font-size: 0.9rem;
  font-weight: 700;
}

.deck-card-row__count--illegal {
  background: rgb(var(--v-theme-error));
  color: #fff;
}

.deck-card-row__count--game-changer {
  background: #ffc107;
  color: #000;
}

.deck-card-row__body {
  min-width: 0;
}

.deck-card-row__top {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 12px;
  align-items: start;
  margin-bottom: 4px;
}

.deck-card-row__bottom {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: baseline;
}

.deck-card-row__name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  overflow: hidden;
  font-weight: 700;
  color: inherit;
}

.deck-card-row__name > span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.deck-card-row__favorite {
  color: #ff5252;
  flex: 0 0 auto;
}

.deck-card-row__status-pill {
  font-size: 0.65rem;
  font-weight: 800;
  padding: 2px 6px;
  border-radius: 6px;
  letter-spacing: 0.05em;
  flex: 0 0 auto;
}

.deck-card-row__status-pill--illegal {
  background: rgb(var(--v-theme-error));
  color: #fff;
}

.deck-card-row__status-pill--game-changer {
  background: #ffc107;
  color: #000;
}

.deck-card-row__mana {
  flex: 0 0 auto;
  font-weight: 700;
  opacity: 0.9;
}

.deck-card-row__actions {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-self: end;
  min-width: 0;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.deck-card-row__buttons {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  max-width: 100%;
}

.deck-card-row__button-glyph {
  font-size: 1rem;
  font-weight: 700;
  line-height: 1;
}

.deck-card-row__bottom {
  font-size: 0.85rem;
  opacity: 0.7;
}

@container (max-width: 290px) {
  .deck-card-row {
    grid-template-columns: 1fr;
  }

  .deck-card-row__count {
    justify-self: start;
    min-width: 52px;
    padding: 0 10px;
  }

  .deck-card-row__top {
    grid-template-columns: 1fr;
  }

  .deck-card-row__actions {
    justify-self: stretch;
    justify-content: space-between;
  }

  .deck-card-row__buttons {
    justify-content: flex-end;
  }

  .deck-card-row__bottom {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
