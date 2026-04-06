<script setup>
import { computed } from "vue";
import { mdiRobotExcitedOutline, mdiHammerWrench, mdiStarFourPoints, mdiFireCircle } from "@mdi/js";

const sprinkleMessages = [
  "Teaching the AI to roast with precision, not cruelty.",
  "Deck slander engine currently being tuned for maximum drama.",
  "Calibrating sass levels for commanders, mana bases, and suspicious card choices.",
];

const featuredMessage = computed(() => sprinkleMessages[1]);
</script>

<template>
  <v-container fluid class="roast-page pa-6 pa-md-10">
    <div class="roast-shell">
      <section class="hero-panel">
        <div class="badge-row">
          <span class="status-pill">
            <v-icon :icon="mdiHammerWrench" size="18" class="mr-2" />
            Under Construction
          </span>
          <span class="mini-pill">Coming Soon</span>
        </div>

        <div class="hero-copy">
          <p class="eyebrow">A.I. Deck Roaster</p>
          <h1 class="hero-title">Your deck is safe for now.</h1>
          <p class="hero-body">
            The roast booth is still being assembled. Soon this page will lovingly inspect your list,
            judge your card choices, and deliver premium cardboard mockery.
          </p>
        </div>

        <div class="message-card">
          <div class="message-icon">
            <v-icon :icon="mdiRobotExcitedOutline" size="30" />
          </div>
          <div>
            <p class="message-label">Current progress</p>
            <p class="message-text">{{ featuredMessage }}</p>
          </div>
        </div>
      </section>

      <section class="details-grid">
        <v-card class="detail-card detail-card--warm" variant="flat">
          <div class="detail-icon">
            <v-icon :icon="mdiFireCircle" size="24" />
          </div>
          <h2>Roasts are brewing</h2>
          <p>
            Expect spicy callouts for greedy keeps, questionable synergies, and decks that swear
            they are "casual."
          </p>
        </v-card>

        <v-card class="detail-card detail-card--soft" variant="flat">
          <div class="detail-icon">
            <v-icon :icon="mdiStarFourPoints" size="24" />
          </div>
          <h2>Cute, but rude</h2>
          <p>
            The goal is playful feedback with enough bite to be fun and enough signal to be useful.
          </p>
        </v-card>
      </section>
    </div>
  </v-container>
</template>

<style scoped>
.roast-page {
  min-height: 100vh;
  background:
    radial-gradient(circle at top left, rgba(255, 214, 165, 0.3), transparent 30%),
    radial-gradient(circle at bottom right, rgba(255, 145, 77, 0.18), transparent 35%),
    linear-gradient(160deg, #2b1612 0%, #120d11 55%, #0b1118 100%);
}

.roast-shell {
  max-width: 1080px;
  margin: 0 auto;
  display: grid;
  gap: 24px;
}

.hero-panel {
  position: relative;
  overflow: hidden;
  padding: 32px;
  border: 1px solid rgba(255, 210, 179, 0.2);
  border-radius: 32px;
  background:
    radial-gradient(circle at top, rgba(255, 244, 214, 0.12), transparent 40%),
    linear-gradient(180deg, rgba(255, 249, 239, 0.08), rgba(255, 255, 255, 0.03));
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(14px);
}

.hero-panel::after {
  content: "";
  position: absolute;
  inset: auto -40px -60px auto;
  width: 220px;
  height: 220px;
  border-radius: 999px;
  background: radial-gradient(circle, rgba(255, 180, 126, 0.28), transparent 65%);
  pointer-events: none;
}

.badge-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 24px;
}

.status-pill,
.mini-pill {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  font-size: 0.82rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.status-pill {
  padding: 10px 16px;
  color: #fff1df;
  background: rgba(255, 166, 102, 0.18);
  border: 1px solid rgba(255, 201, 158, 0.35);
}

.mini-pill {
  padding: 10px 14px;
  color: #ffd8b5;
  background: rgba(255, 255, 255, 0.06);
}

.hero-copy {
  max-width: 700px;
}

.eyebrow {
  margin-bottom: 10px;
  color: #ffcf9f;
  font-size: 0.9rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.hero-title {
  margin: 0;
  color: #fff8f0;
  font-size: clamp(2.7rem, 5vw, 4.8rem);
  line-height: 0.95;
  letter-spacing: -0.04em;
}

.hero-body {
  margin-top: 18px;
  max-width: 640px;
  color: rgba(255, 240, 225, 0.86);
  font-size: 1.05rem;
  line-height: 1.7;
}

.message-card {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 16px;
  align-items: center;
  margin-top: 28px;
  padding: 18px;
  border-radius: 24px;
  background: rgba(16, 22, 28, 0.45);
  border: 1px solid rgba(255, 232, 208, 0.12);
}

.message-icon {
  display: grid;
  place-items: center;
  width: 56px;
  height: 56px;
  border-radius: 18px;
  color: #fff1de;
  background: linear-gradient(160deg, #ff8d5c, #ffb56b);
  box-shadow: 0 14px 30px rgba(255, 141, 92, 0.28);
}

.message-label {
  margin: 0 0 6px;
  color: #ffc896;
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.message-text {
  margin: 0;
  color: #fff4ea;
  font-size: 1rem;
  line-height: 1.6;
}

.details-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 20px;
}

.detail-card {
  min-height: 220px;
  padding: 24px;
  border-radius: 28px;
  color: #fff7ef;
  border: 1px solid rgba(255, 236, 220, 0.12);
}

.detail-card--warm {
  background: linear-gradient(180deg, rgba(113, 39, 22, 0.92), rgba(67, 24, 18, 0.92));
}

.detail-card--soft {
  background: linear-gradient(180deg, rgba(33, 41, 58, 0.94), rgba(24, 27, 43, 0.94));
}

.detail-icon {
  display: inline-grid;
  place-items: center;
  width: 48px;
  height: 48px;
  margin-bottom: 18px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.12);
}

.detail-card h2 {
  margin: 0 0 12px;
  font-size: 1.45rem;
}

.detail-card p {
  margin: 0;
  color: rgba(255, 244, 235, 0.82);
  line-height: 1.7;
}

@media (max-width: 900px) {
  .details-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .hero-panel {
    padding: 24px;
    border-radius: 24px;
  }

  .hero-title {
    font-size: 2.5rem;
  }

  .message-card {
    grid-template-columns: 1fr;
  }
}
</style>
