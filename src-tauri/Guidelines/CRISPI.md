
# CRISPI Deck Evaluation Framework

## Overview
CRISPI is a rubric-driven system to evaluate a Commander deck’s practical power by scoring five measurable dimensions:

- **C**onsistency
- **R**esilience
- **I**nteraction
- **S**peed
- **P**ivotability

Each category is scored **0–5** and summed for a raw CRISPI score (0–25).

An **Average Mana Value (AMV) modifier** is then applied to slightly amplify or dampen the score based on the deck’s mana curve.

---

## C — Consistency
Measures how reliably the deck executes its main strategy.

**Considerations:**
- Tutors
- Redundant combo pieces
- Card selection
- Mana smoothing and curve discipline
- Commander as engine vs passenger

---

## R — Resilience
Measures how well the deck survives disruption.

**Considerations:**
- Protection
- Recursion
- Board wipe recovery
- Independence from commander
- Resistance to hate pieces

---

## I — Interaction
Measures the deck’s ability to disrupt opponents.

**Considerations:**
- Spot removal efficiency
- Stack interaction
- Stax / hate pieces
- Instant-speed responses

---

## S — Speed
Measures how quickly the deck can win or lock the game.

**Considerations:**
- Fast mana density
- Average mana value (AMV)
- Earliest deterministic win turn
- Compact combo requirements

---

## P — Pivotability
Measures how easily the deck can change plans mid-game.

**Considerations:**
- Multiple win conditions
- Switching between combo, control, and value
- Commander enabling alternate lines
- Cards with multiple roles

---

## Scoring Rubric

| Score | Meaning |
|------:|---------|
| 0 | Virtually absent |
| 1 | Very light presence |
| 2 | Present but not core |
| 3 | Strong component |
| 4 | Major strength |
| 5 | Defining characteristic |

---

## Raw CRISPI Score
RawCRISPI = C + R + I + S + P

---

## AMV Adjustment

MaxTargetAMV = 5.0

M = 1 + ((MaxTargetAMV - DeckAMV) / MaxTargetAMV) * 0.10

Clamp M between 0.90 and 1.10

FinalCRISPI = RawCRISPI * M

---

## Final CRISPI Interpretation

| Final Score | Interpretation |
|-------------|----------------|
| 0–8 | Casual / Battlecruiser |
| 9–14 | Focused / Synergistic |
| 15–19 | High Power |
| 20–23 | Fringe cEDH |
| 24–25+ | cEDH Optimized |
