<script setup>
import {runCommand} from "../api/commands.js";
import {greeting} from "../api/commands.js";


</script>

<template>
<v-container>
  This is the Home page
  <!-- v Command Wrapper v Test  -->
  <v-btn @click="runCommand('string from command.ts wrapper')">Test Command</v-btn>
  <div>{{ greeting }}</div>
  <!-- ^ Command Wrapper Test ^ -->
</v-container>
</template>

<style scoped>

</style>
