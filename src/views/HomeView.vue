<script setup>
import { ref } from "vue";
import { runTestCommand } from "../api/commands.js";

const testGreetingInput = ref("string from HomeView.vue");
const testCommandResult = ref("");
const testCommandError = ref("");

async function handleTestCommand() {
  testCommandError.value = "";

  try {
    testCommandResult.value = await runTestCommand(testGreetingInput.value);
  } catch (error) {
    testCommandResult.value = "";
    testCommandError.value = String(error);
  }
}
</script>

<template>
  <v-container class="home-page">
    <h1>Home</h1>
    <p>This page includes a simple frontend-to-Rust command example.</p>

    <section class="command-reference">
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
  </v-container>
</template>

<style scoped>
.home-page {
  display: grid;
  gap: 16px;
  color: #132032;
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
</style>
