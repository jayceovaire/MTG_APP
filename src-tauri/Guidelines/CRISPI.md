
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

The evaluation also includes a **Deck Archetype Label** (Turbo, Midrange, Stax, or Commander Engine).

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
- Creature or Artifact with at least one role and a non-tapping activation

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

## Deck Archetype Detection

🔶 **New Section — Step 3: Deck Archetype Detection (Required)**

Before scoring, detect the deck's archetype to apply specialized adjustments.

### Archetype Signals

- **Turbo Signal:** `ExplosiveManaPoints + ExplosiveDrawPoints` (Floor: 18.0)
- **Midrange Signal:** `ConsistencyWeighted + EngineWeighted + DrawWeighted`
- **Stax Signal:** `WeightedStaxSum` (Non-land = 1.0, Land = 0.3) (Floor: 15.0)
- **Voltron Signal:** `WeightedVoltronSum` (Equipment/Aura count) (Floor: 8.0)
- **Group Hug Signal:** `WeightedGroupHugSum` (Floor: 8.0)

### Classification Rule

1. If multiple signals are above their floors, the **highest signal wins**.
2. If no signal is above its floor, the deck defaults to **Midrange**.
3. **Commander Engine** is a secondary classification triggered if the Commander provides significant card advantage (>1.0 signal for Partners) AND the Turbo signal > 12.0, provided it's the highest signal.
4. If `TurboSignal > 12.0` and it's higher than `MidrangeSignal`, it can also be classified as **Turbo**.

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

🔶 **New Rule — Win Package Density Adjustment**  
Decks with high win condition density (`win_package_density > 0.08`) receive `Consistency +1` (capped at 5). This rewards Turbo decks that favor redundancy over tutors.

---

## R — Resilience (0–5)

Measures recovery and protection from disruption.

### Rules
- Protection must be ≤2 MV to count fully
- Recursion ≤3 MV counts fully
- If the deck **cannot function without its commander**, **Resilience is capped at 2**

### Scoring Thresholds (Weighted Value)

| Score | Weighted Value |
|---:|---|
| 5 | ≥ 10.0 |
| 4 | ≥ 7.0 |
| 3 | ≥ 4.0 |
| 2 | ≥ 1.5 |
| 1 | < 1.5 |
| 0 | 0.0 |

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

CRISPI uses **Dual-Axis Speed** to evaluate both efficiency and explosiveness.

Speed = max(EfficiencySpeed, ExplosiveSpeed)

### 1. Efficiency Speed (Traditional)

Efficiency Speed is determined by the deck's win turn and its fast mana base.

**Earliest Deterministic Win Turn (goldfish)**

| Turn | Speed Score |
|---:|---:|
| ≤3 | 5 |
| 4 | 4 |
| 5 | 3 |
| 6 | 2 |
| ≥7 | 1 |

**Fast Mana Gate (caps Efficiency Speed)**

| Fast Mana Count | Speed Cap |
|---:|---:|
| 0–3 | 2 |
| 4–7 | 3 |
| 8+ | No cap |

Fast mana includes 0–1 MV rocks, rituals, and mana-positive artifacts.

### 2. Explosive Speed (Velocity)

Explosive Speed measures the deck's ability to generate massive bursts of mana or card draw in turns 1–2.

**Mana Velocity Calculation:**
`ManaVelocity = (FastManaCount * 0.8) + ExplosiveManaPoints`

**Explosive Mana Points:**
- **Ritual (≥3 mana):** 2.5 pts
- **Treasure Burst:** 2.0 pts
- **Sacrifice Mana:** 1.8 pts
- **Cost Reduction:** 1.5 pts
- **One-Shot Fast Mana:** 1.8 pts (includes burst sacrifice for mana/treasures without tapping)

**Draw Velocity Calculation:**
`DrawVelocity = (PremiumDrawCount * 0.8) + ExplosiveDrawPoints`

**Explosive Draw Points:**
- **Wheel Effect:** 2.5 pts
- **Mass Draw (≥3 cards):** 2.0 pts
- **Burst Draw:** 1.5 pts

**Explosive Speed Score:**

| Velocity (Mana or Draw) | Speed Score |
|---:|---:|
| ≥18.0 | 5 |
| ≥13.0 | 4 |
| ≥9.0 | 3 |
| ≥5.0 | 2 |
| <5.0 | 1 |

Final Speed Score = max(EfficiencySpeed, ExplosiveSpeed)

---

## P — Pivotability (0–5)

Measures ability to change plans mid-game.

> To score **≥3**, at least **8 cards** must serve roles in **multiple win lines**.

### Considerations
- Multiple win conditions sharing pieces
- Cards with multiple roles
- Commander enabling alternate lines

---

## 🔶 Infinite Combo Detection (New)

CRISPI detects two and three-card infinite combos in the decklist and adjusts the score and bracket accordingly.

### Multiplier Rules

- **Max Bonus (per combo):** **+0.20**
- **Base Bonus:** **+0.02** for combos with total combined mana value ≤3 (two-card) or ≤4 (three-card).
- **Tutor Scaling:** **+0.02** per tutor (capped at **+0.15**).
- **Mana Value Scaling:** Penalty of **-0.01** per MV for every point above **3** (two-card) or **4** (three-card) (total combined cost).
- **Speed Scaling:** Penalty of **-0.02** for each slow component (Sorcery-speed spells, or Tap abilities on creatures without haste).
- **Minimum Bonus:** Every valid infinite combo grants at least **+0.02** (×1.02).
- **Additive Bonus:** If multiple separate combos are detected, their bonuses are combined additively: `1.0 + sum(bonuses)`.
- **Total Cap:** Total combo multiplier is capped at **×1.25** (to allow scaling with multiple combos).
- **Score Cap:** The final `Total Score` is capped at **25.0**.

### Final Multiplier Calculation

The AMV multiplier and Combo multiplier are combined into a `Final Multiplier`:
`FinalMultiplier = AMVMultiplier * ComboMultiplier`

`FinalCRISPI = min(RawCRISPI * FinalMultiplier, 25.0)`

### Bracket Override

Detecting any valid infinite combo overrides lower brackets up to **Bracket 4**.

### Two-Card Combo List (Examples)

| Card A | Card B | Effect |
|---|---|---|
| Demonic Consultation | Thassa's Oracle | win |
| Exquisite Blood | Sanguine Bond | lifegain, damage |
| Tainted Pact | Thassa's Oracle | win |
| Dramatic Reversal | Isochron Scepter | mana |
| Dualcaster Mage | Twinflame | ETB, LTB |
| Niv-Mizzet, Parun | Curiosity | draw, damage |
| Basalt Monolith | Forsaken Monument | mana |
| Bruvac the Grandiloquent | Maddening Cacophony | mill |
| Kiki-Jiki, Mirror Breaker | Zealous Conscripts | ETB |
| Godo, Bandit Warlord | Helm of the Host | ETB |
| Karn, the Great Creator | Mycosynth Lattice | lock |

*Note: Prerequisites must be met for a combo to be valid (e.g., sufficient non-land count for complex setups).*

---

## Structural Floors (Override Category Minimums)

🔶 **New Section — Structural Floors (Override Category Minimums)**

These are applied after scoring but before AMV modifier.

### Structural Floors

| Condition | Category Floor |
|---|---|
| Commander is repeatable card advantage engine (e.g. Tymna the Weaver) | Consistency ≥4, Pivotability ≥3 |
| 8+ free interaction spells (e.g. Force of Will) | Interaction = 5, Resilience ≥4 |
| Compact ≤3 card deterministic win package | Speed = 5, Consistency +1 |
| 10+ fast mana pieces | Speed = 5 |
| 5+ premium ≤2 MV tutors | Consistency = 5 |

### Archetype-Aware Adjustments

| Archetype | Score Adjustments |
|---|---|
| **Turbo** | `Consistency >= 4`, `Pivotability >= 3` |
| **Stax** | `Interaction >= 4`, `Resilience >= 4` |
| **Commander Engine** | `Consistency >= 4`, `Resilience >= 3`, `Pivotability >= 3` |
| **Voltron** | `Resilience >= 4`, `Pivotability >= 2` |
| **Group Hug** | `Consistency >= 3`, `Pivotability >= 3` |
| **Midrange** | `Consistency >= 3`, `Resilience >= 3`, `Interaction >= 3`, `Pivotability >= 2` |

Multiple floors and archetype adjustments may apply. Scores are capped at 5.

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
| ≤1.3 | 1.12 |
| 1.31–1.6 | 1.06 |
| 1.61–2.0 | 1.02 |
| 2.01–2.4 | 1.01 |
| 2.41–2.8 | 1.00 |
| 2.81–3.0 | 0.92 |
| 3.01–3.4 | 0.85 |
| 3.41–3.8 | 0.65 |
| >3.8 | 0.50 |
|
FinalCRISPI = min(RawCRISPI * FinalMultiplier, 25.0)

---

## Final CRISPI Interpretation

| Final Score | Interpretation |
|---:|---|
| 0–8 | Casual / Battlecruiser |
| 9–14 | Focused / Synergistic |
| 15–18 | High Power |
| 19–22 | Fringe cEDH |
| 23+ | cEDH Optimized |

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
| **Infinite Combo Detected** | **Min Bracket 4** |
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