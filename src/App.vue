<script setup>
import SideNavBar from "./components/SideNavBar.vue";

</script>

<template>
  <v-app>
    <SideNavBar />
    <v-main>
      <router-view />
    </v-main>
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
