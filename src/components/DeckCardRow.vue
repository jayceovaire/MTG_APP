<script setup>
import { mdiAccountMultiplePlusOutline, mdiCloseCircleOutline, mdiCrownOutline, mdiDotsHorizontal } from "@mdi/js";
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
});

const emit = defineEmits(["add-copy", "remove-copy", "set-commander", "set-partner", "remove-commander", "remove-partner"]);

function buildTypeLine(card) {
  const superType = Array.isArray(card.super_type) ? card.super_type.join(" ") : "";
  const cardType = Array.isArray(card.card_type) ? card.card_type.join(" ") : "Card";
  const subType = Array.isArray(card.sub_type) && card.sub_type.length > 0 ? ` - ${card.sub_type.join(" ")}` : "";
  return `${superType} ${cardType}${subType}`.trim();
}
</script>

<template>
  <article class="deck-card-row">
    <div class="deck-card-row__count">{{ quantity }}x</div>
    <div class="deck-card-row__body">
      <div class="deck-card-row__top">
        <span class="deck-card-row__name">{{ card.name }}</span>
        <div class="deck-card-row__actions">
          <ManaText class="deck-card-row__mana" :text="card.mana_cost || ''" empty-text="-" :cost="true" />
          <div v-if="editable" class="deck-card-row__buttons">
            <v-btn size="x-small" variant="outlined" icon @click.stop="$emit('remove-copy')">
              <span class="deck-card-row__button-glyph">-</span>
            </v-btn>
            <v-btn size="x-small" variant="outlined" icon @click.stop="$emit('add-copy')">
              <span class="deck-card-row__button-glyph">+</span>
            </v-btn>
            <v-menu location="bottom end">
              <template #activator="{ props: menuProps }">
                <v-btn v-bind="menuProps" size="x-small" variant="outlined" icon @click.stop>
                  <v-icon :icon="mdiDotsHorizontal" size="14"></v-icon>
                </v-btn>
              </template>
              <v-list density="compact">
                <v-list-item
                  :disabled="!canSetCommander"
                  title="Set as Commander"
                  @click.stop="$emit('set-commander')"
                >
                  <template #prepend>
                    <v-icon :icon="mdiCrownOutline" size="16"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  :disabled="!canSetPartner"
                  title="Set as Partner"
                  @click.stop="$emit('set-partner')"
                >
                  <template #prepend>
                    <v-icon :icon="mdiAccountMultiplePlusOutline" size="16"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  :disabled="!canRemovePartner"
                  title="Remove as Partner"
                  @click.stop="$emit('remove-partner')"
                >
                  <template #prepend>
                    <v-icon :icon="mdiCloseCircleOutline" size="16"></v-icon>
                  </template>
                </v-list-item>
                <v-list-item
                  :disabled="!canRemoveCommander"
                  title="Remove as Commander"
                  @click.stop="$emit('remove-commander')"
                >
                  <template #prepend>
                    <v-icon :icon="mdiCloseCircleOutline" size="16"></v-icon>
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
  border: 1px solid rgba(14, 21, 30, 0.08);
  background: rgba(255, 255, 255, 0.8);
  container-type: inline-size;
}

.deck-card-row__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 36px;
  border-radius: 10px;
  background: #0f1724;
  color: #f5f7fb;
  font-size: 0.9rem;
  font-weight: 700;
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
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 700;
  color: #142033;
}

.deck-card-row__mana {
  flex: 0 0 auto;
  font-weight: 700;
  color: #43526b;
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
  color: #5f6f86;
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
