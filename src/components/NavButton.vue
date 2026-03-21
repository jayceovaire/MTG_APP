<script setup>
import { useRouter } from 'vue-router'

const router = useRouter()


const props = defineProps({
  to: {
    type: [String, Object],
    required: true
  },
  width: String,

});

function navigate() {
  router.push(props.to)
}
</script>

<template>
  <v-btn :width="props.width" @click="navigate">
    <slot />
  </v-btn>
</template>

<style scoped>

</style>
