<script setup>
import { computed } from "vue";
import { tokenizeManaText } from "../utils/manaSymbols.js";

const props = defineProps({
  text: {
    type: String,
    default: "",
  },
  emptyText: {
    type: String,
    default: "",
  },
  cost: {
    type: Boolean,
    default: false,
  },
  multiline: {
    type: Boolean,
    default: false,
  },
});

const segments = computed(() => tokenizeManaText(props.text));
</script>

<template>
  <span class="mana-text" :class="{ 'mana-text--multiline': multiline }">
    <template v-if="segments.length > 0">
      <template v-for="(segment, index) in segments" :key="`${segment.type}-${index}`">
        <span v-if="segment.type === 'text'">{{ segment.value }}</span>
        <i
          v-else-if="segment.className"
          :class="['ms', segment.className, { 'ms-cost': cost }]"
          :title="segment.raw"
          aria-hidden="true"
        ></i>
        <span v-else>{{ segment.raw }}</span>
      </template>
    </template>
    <template v-else>
      {{ emptyText }}
    </template>
  </span>
</template>

<style scoped>
.mana-text {
  display: inline;
}

.mana-text :deep(.ms) {
  vertical-align: middle;
}

.mana-text :deep(.ms-cost) {
  width: 1.45em;
  height: 1.45em;
  line-height: 1.45em;
  font-size: 0.92em;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  vertical-align: middle;
}

.mana-text :deep(.ms-cost.ms-wu),
.mana-text :deep(.ms-cost.ms-wb),
.mana-text :deep(.ms-cost.ms-ub),
.mana-text :deep(.ms-cost.ms-ur),
.mana-text :deep(.ms-cost.ms-br),
.mana-text :deep(.ms-cost.ms-bg),
.mana-text :deep(.ms-cost.ms-rw),
.mana-text :deep(.ms-cost.ms-rg),
.mana-text :deep(.ms-cost.ms-gw),
.mana-text :deep(.ms-cost.ms-gu),
.mana-text :deep(.ms-cost.ms-2w),
.mana-text :deep(.ms-cost.ms-2u),
.mana-text :deep(.ms-cost.ms-2b),
.mana-text :deep(.ms-cost.ms-2r),
.mana-text :deep(.ms-cost.ms-2g),
.mana-text :deep(.ms-cost.ms-cw),
.mana-text :deep(.ms-cost.ms-cu),
.mana-text :deep(.ms-cost.ms-cb),
.mana-text :deep(.ms-cost.ms-cr),
.mana-text :deep(.ms-cost.ms-cg),
.mana-text :deep(.ms-cost.ms-wup),
.mana-text :deep(.ms-cost.ms-wbp),
.mana-text :deep(.ms-cost.ms-ubp),
.mana-text :deep(.ms-cost.ms-urp),
.mana-text :deep(.ms-cost.ms-brp),
.mana-text :deep(.ms-cost.ms-bgp),
.mana-text :deep(.ms-cost.ms-rwp),
.mana-text :deep(.ms-cost.ms-rgp),
.mana-text :deep(.ms-cost.ms-gwp),
.mana-text :deep(.ms-cost.ms-gup) {
  width: 1.5em;
  height: 1.5em;
}

.mana-text--multiline {
  white-space: pre-line;
}
</style>
