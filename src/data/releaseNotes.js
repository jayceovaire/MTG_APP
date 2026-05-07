export const releaseNotes = [
  {
    version: "0.1.7",
    publishedOn: "2026-05-06",
    headline: "Enhanced combo data and improved metadata mapping.",
    bullets: [
      "Fixed missing combo steps and instructions in the Power Calculator and Card Viewer.",
      "Implemented a robust fallback mechanism for sidecar data to ensure full instructions are always shown.",
      "Integrated ManaText component for correct rendering of mana symbols and action icons in combo steps.",
      "Improved combo deduplication logic and increased sidecar connection resilience.",
      "Bumped application version to 0.1.7 and updated signing configuration."
    ],
  },
  {
    version: "0.1.6",
    publishedOn: "2026-04-29",
    headline: "Rebranding, combo integration, and advanced synergy metrics.",
    bullets: [
      "Rebranded application to 'Anura' and updated developer to 'Aesir Software'.",
      "Integrated Commander Spellbook Sidecar for advanced infinite combo detection (2-card and 3-card).",
      "Introduced Integration Metrics to measure card-level synergy and deck network density.",
      "Added detection for Land Destruction and Mass Land Destruction roles.",
      "Improved card viewer and UI layouts across collection and deck builders.",
      "Standardized User-Agent strings and local data persistence paths."
    ],
  },
  {
    version: "0.1.5",
    publishedOn: "2026-04-16",
    headline: "Sidecar integration fixes and combo detection improvements.",
    bullets: [
      "Fixed sidecar process termination on application exit.",
      "Improved combo detection logic to correctly parse sidecar API responses.",
      "Standardized sidecar logging to reduce terminal clutter.",
      "Bumped application version to 0.1.5."
    ],
  },
  {
    version: "0.1.4",
    publishedOn: "2026-04-15",
    headline: "Storm support, stronger deck metrics, and editor workflow improvements.",
    bullets: [
      "Added Storm archetype support, including Storm role scoring and the related info-page updates.",
      "Expanded deck analysis with integration metrics and explicit three-card combo support.",
      "Refined ramp and ritual role scoring to produce more accurate deck evaluations.",
      "Added drag-and-drop reordering for decks and packages, plus a rename action in the deck editor.",
      "Introduced official dark-mode and light-mode color schemes.",
      "New decks now appear at the front of the deck library."
    ],
  }
];
