export const releaseNotes = [
  {
    version: "0.1.3",
    publishedOn: "2026-04-08",
    headline: "Deck analysis and updater stability improvements.",
    bullets: [
      "Improved deck scoring with stronger handling for color fixing, tutors, mana curve pressure, turbo lines, and consistency signals.",
      "Fixed several regex-driven card classification issues affecting ramp, treasure, removal, draw, infect, and proliferation detection.",
      "Adjusted bracket scoring for edge cases, including three-card combo decks, false-positive cEDH flags, and game changer amplification limits.",
      "Split core logic and tests into more maintainable modules to make the scoring system easier to extend safely.",
      "Added the updater flow and follow-up bug fixes so in-app updates are more reliable."
    ],
  }
];
