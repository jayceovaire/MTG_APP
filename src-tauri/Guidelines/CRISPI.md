
# CRISPI Deck Evaluation Framework (v2 — Efficiency Corrected)

CRISPI evaluates a Commander deck’s **practical competitive power** using five dimensions:

- **C**onsistency
- **R**esilience
- **I**nteraction
- **S**peed
- **P**ivotability

Each category is scored **0–5** using **strict efficiency rules**.

> CRISPI measures **how efficiently** a deck executes its plan — not whether the pieces merely exist.

RawCRISPI = C + R + I + S + P  (0–25)

A **non-linear Average Mana Value (AMV) modifier** is then applied.

---

## Global Efficiency Rule (applies to ALL categories)

🔶 **New Global Rule — Roles Cannot Be Counted Raw**

CRISPI scoring MUST NOT use raw role counts.
All roles must be weighted by card quality before contributing to any category score.

🔶 **New Section — Step 1: Card Quality Classification (Mandatory First Pass)**

Before roles are inferred, every card must be classified into a Quality Tier.

### Quality Tiers

| Tier | Meaning | Weight |
|---|---|---|
| **Premium** | cEDH staple rate, free spell, fast mana, ≤2 MV premium effect | 1.0 |
| **Efficient** | Playable but not staple rate | 0.6 |
| **Slow** | Precon / casual rate | 0.2 |

**A card is Premium if ANY apply:**
- Mana value ≤2 and is commonly played in cEDH
- Free spell or alternate cost interaction
- Produces mana ≥ its cost (fast mana, rituals, LED, Moxen, Crypt, Petal)
- Unrestricted tutor ≤2 MV (e.g. Demonic Tutor)
- ≤2 MV instant-speed premium interaction (e.g. Swords to Plowshares)
- Compact combo piece ≤3 MV (e.g. Thassa's Oracle, Underworld Breach)

If none apply and MV ≥4 → **Slow**

## 🔶 Special Rule — Land Quality Classification (CRITICAL)

Lands MUST NOT automatically be considered **Premium**.  
Land quality must be evaluated separately from other card types.

### Land Quality Tiers

| Land Characteristics | Quality | Weight |
|---|---|---:|
| Enters untapped **and** produces more than one mana **OR** more than one color **OR** provides tutor / recursion / interaction effect | **Premium** | 1.0 |
| Enters tapped **but** has a strong effect (multi-mana burst, tutor, recursion, or interaction) | **Efficient** | 0.6 |
| Enters untapped and only produces a single color with no additional effect | **Efficient** | 0.6 |
| Enters tapped and only produces mana (gain life, scry, ping, or no extra effect) | **Slow / Inefficient** | 0.2 |

### Rule

> Lands must be classified by **tempo and utility**, not by being lands.  
> Simple or basic lands that only produce one color are **not Premium**.

🔶 **New Rule — Roles Are Multiplied by Quality Weight**

When a card contributes to any CRISPI category:

`role_value = role_presence × quality_weight`

This replaces all raw counting.

---

## Deck Structural Metrics

🔶 **New Section — Step 2: Deck Structural Metrics (Required)**

Before category scoring, compute these deck-wide metrics:

- **Fast mana count**
- **Free interaction count**
- **Premium tutor count**
- **Average mana value (AMV)**
- **Presence of compact 2-card win package**
- **Whether commander provides repeatable card advantage**

These metrics are REQUIRED inputs to scoring.

---

## C — Consistency (0–5)

Measures how reliably the deck assembles its plan.

### Tutor Quality Weights

| Tutor Type | Value |
|---|---|
| ≤2 MV, searches any card (e.g. **:contentReference[oaicite:0]{index=0}**) | 1.0 |
| 3–4 MV, any card | 0.5 |
| 4+ MV or conditional (e.g. **:contentReference[oaicite:1]{index=1}**) | 0.25 |

### Additional Factors
- Redundant combo pieces (≤2 MV)
- Cheap card selection (≤2 MV)
- Commander as engine (not passenger)
- Mana smoothing via cheap rocks/dorks

### Scoring Guide

| Score | Requirement |
|---:|---|
| 5 | 8+ weighted tutor value + strong redundancy |
| 4 | 5–7 weighted tutor value |
| 3 | 3–4 weighted tutor value |
| 2 | 1–2 weighted tutor value |
| 1 | Minimal search/selection |
| 0 | None |

---

## R — Resilience (0–5)

Measures recovery and protection from disruption.

### Rules
- Protection must be ≤2 MV to count fully
- Recursion ≤3 MV counts fully
- If the deck **cannot function without its commander**, **Resilience is capped at 2**

### Considerations
- Protection, recursion, wipe recovery
- Independence from commander
- Resistance to hate/stax

---

## I — Interaction (0–5)

Measures ability to disrupt opponents **at competitive speed**.

> To score **≥3**, the deck must contain **at least 6 pieces** of ≤2 MV instant-speed interaction, including stack interaction.

### Interaction Quality

| Type | Full Value |
|---|---|
| ≤2 MV instant removal (e.g. **Swords to Plowshares**) | Yes |
| Premium Stax pieces (e.g. **Thalia, Guardian of Thraben**) | Yes |
| 3 MV instant removal (e.g. **Beast Within**) | Half |
| 4+ MV removal (e.g. **Utter End**) | Zero |

---

## S — Speed (0–5)

Measures how quickly the deck can **deterministically win**.

🔶 **New Rule — Speed Is Determined ONLY by Win Turn + Fast Mana**

Speed score is NOT influenced by roles. It is computed strictly from:
- Earliest deterministic goldfish win turn
- Fast mana count cap

### Earliest Deterministic Win Turn (goldfish)

| Turn | Speed Score |
|---:|---:|
| ≤3 | 5 |
| 4 | 4 |
| 5 | 3 |
| 6 | 2 |
| ≥7 | 1 |

### Fast Mana Gate (caps Speed)

| Fast Mana Count | Speed Cap |
|---:|---:|
| 0–3 | 2 |
| 4–7 | 3 |
| 8+ | No cap |

Fast mana includes 0–1 MV rocks, rituals, and mana-positive artifacts.

---

## P — Pivotability (0–5)

Measures ability to change plans mid-game.

> To score **≥3**, at least **8 cards** must serve roles in **multiple win lines**.

### Considerations
- Multiple win conditions sharing pieces
- Cards with multiple roles
- Commander enabling alternate lines

---

## Structural Floors (Override Category Minimums)

🔶 **New Section — Structural Floors (Override Category Minimums)**

These are applied after scoring but before AMV modifier.

| Condition | Category Floor |
|---|---|
| Commander is repeatable card advantage engine (e.g. Tymna the Weaver) | Consistency ≥4, Pivotability ≥4 |
| 8+ free interaction spells (e.g. Force of Will) | Interaction = 5, Resilience ≥4 |
| Compact ≤2 card deterministic win package | Speed = 5, Consistency +1 |
| 10+ fast mana pieces | Speed = 5 |
| 5+ premium ≤2 MV tutors | Consistency = 5 |

Multiple floors may apply.

---

## Raw CRISPI Score

RawCRISPI = C + R + I + S + P

---

## Non-Linear AMV Modifier (Critical)

No cEDH deck has AMV above 3.0. High AMV must be heavily penalized.

🔶 **New Rule — AMV Is Applied After Structural Floors**

AMV multiplier is the final step after all category floors and adjustments.

| Deck AMV | Multiplier (M) |
|---:|---:|
| ≤2.2 | 1.12 |
| 2.21–2.6 | 1.06 |
| 2.61–3.0 | 1.00 |
| 3.01–3.4 | 0.88 |
| 3.41–3.8 | 0.72 |
| >3.8 | 0.55 |
|
FinalCRISPI = RawCRISPI × M

---

## Final CRISPI Interpretation

| Final Score | Interpretation |
|---:|---|
| 0–8 | Casual / Battlecruiser |
| 9–14 | Focused / Synergistic |
| 15–19 | High Power |
| 20–23 | Fringe cEDH |
| 24+ | cEDH Optimized |

---

## Bracket System (1–5)

### Base Bracket — Game Changers (GC)

| GC Count | Base Bracket |
|---:|---:|
| 0 | 2 |
| 1–3 | 3 |
| >3 | 4 |

### Refinements

| Condition | Final Bracket |
|---|---|
| Base Bracket 2, CRISPI 0–8, AMV >3.5 | 1 |
| FinalCRISPI ≥24 | 5 |

---

## Key Principle

🔶 **New Principle**

CRISPI measures card quality density and deck structure, not card purpose frequency.
This rule is what allows Blue Farm to reach 25 and precons to fall correctly.

CRISPI rewards:
- Low mana value
- Fast execution
- Efficient tutors
- Cheap interaction
- Overlapping combo pieces

CRISPI penalizes:
- Slow cards
- Commander dependence
- Expensive tutors/removal
- High mana curves

This prevents preconstructed decks from falsely scoring as competitive decks.