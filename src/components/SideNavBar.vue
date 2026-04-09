<script setup>
import { ref } from "vue";
import {
  mdiHomeOutline,
  mdiCardsOutline,
  mdiPackageVariantClosed,
  mdiViewGridOutline,
  mdiCalculatorVariantOutline,
  mdiFire,
  mdiCog,
  mdiInformationOutline,
} from "@mdi/js";

const isRail = ref(true);
const primaryItems = [
  { title: "Home", to: "/", icon: mdiHomeOutline },
  { title: "Decks", to: "/deck-library", icon: mdiCardsOutline },
  { title: "Packages", to: "/package-library", icon: mdiPackageVariantClosed },
  { title: "Collection", to: "/collection", icon: mdiViewGridOutline },
  { title: "Tools", to: "/power-calculator", icon: mdiCalculatorVariantOutline },
  { title: "Roast", to: "/roast", icon: mdiFire },

];

const bottomItems = [
  { title: "Information", to: "/info", icon: mdiInformationOutline },
  { title: "Settings", to: "/settings", icon: mdiCog },
];

function toggleDrawerWidth() {
  isRail.value = !isRail.value;
}
</script>

<template>
  <v-navigation-drawer
    permanent
    :rail="isRail"
    :width="150"
    :rail-width="56"
    :class="['side-nav', { 'side-nav--rail': isRail }]"
  >
    <div class="side-nav__top">
      <v-btn
        size="x-small"
        variant="text"
        min-width="28"
        width="28"
        height="28"
        aria-label="Toggle navigation"
        @click="toggleDrawerWidth"
      >
        <span class="hamburger-icon" aria-hidden="true">
          <span></span>
          <span></span>
          <span></span>
        </span>
      </v-btn>
    </div>

    <v-list nav density="compact" class="side-nav__list" color="primary">
      <v-list-item
        v-for="item in primaryItems"
        :key="item.title"
        :to="item.to"
        :disabled="!item.to"
        :title="isRail ? undefined : item.title"
        rounded="lg"
      >
        <template #prepend>
          <v-icon :icon="item.icon" size="18" class="side-nav__icon"/>
        </template>
      </v-list-item>
    </v-list>

    <v-spacer />

    <v-list nav density="compact" class="side-nav__list side-nav__list--bottom" color="primary">
      <v-list-item
        v-for="item in bottomItems"
        :key="item.title"
        :to="item.to"
        :title="isRail ? undefined : item.title"
        rounded="lg"
      >
        <template #prepend>
          <v-icon :icon="item.icon" size="18" class="side-nav__icon"/>
        </template>
      </v-list-item>
    </v-list>
  </v-navigation-drawer>
</template>

<style scoped>
.side-nav {
  background: rgb(var(--v-theme-surface));
  border-right: 1px solid rgba(var(--v-border-color), 0.12);
}

.side-nav__top {
  display: flex;
  justify-content: flex-start;
  padding: 10px 12px 8px;
}

.side-nav__list {
  padding: 2px 6px;
}
.side-nav__icon {
  opacity: 0.7;
}

:deep(.v-list-item--active .side-nav__icon) {
  opacity: 1;
}

:deep(.v-list-item--active) {
  background-color: rgba(var(--v-theme-primary), 0.08) !important;
}
.side-nav__list--bottom {
  margin-bottom: 10px;
}

:deep(.v-list-item) {
  margin: 2px 0;
  min-height: 38px;
}

:deep(.v-list-item__prepend > .v-icon) {
  opacity: 0.9;
}

:deep(.v-list-item-title) {
  font-size: 13px;
  font-weight: 600;
}

.side-nav--rail :deep(.v-list-item-title) {
  display: none;
}

.side-nav__glyph {
  width: 18px;
  display: inline-flex;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  opacity: 0.9;
}

.hamburger-icon {
  display: inline-flex;
  flex-direction: column;
  justify-content: center;
  gap: 2px;
}

.hamburger-icon span {
  display: block;
  width: 12px;
  height: 2px;
  border-radius: 999px;
  background-color: currentColor;
}
</style>
