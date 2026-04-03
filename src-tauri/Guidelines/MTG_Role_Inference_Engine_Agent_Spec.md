# MTG Role Inference Engine --- Agent Build Spec

## Purpose

Build a **runtime role inference engine** that classifies a card's
functional roles (**Ramp, Draw, Removal, Tutor, Protection, Land,
Fixing, Engine, Wincon, Stax**) using only data already present from the
Scryfall API:

-   `oracle_text`
-   `type_line`
-   `mana_cost`
-   `colors`
-   `cmc`

❗ Do NOT create or rely on a pre-tagged database of cards.\
❗ Only analyze the \~100 cards in the active deck including the commander / partners. 

------------------------------------------------------------------------

## High Level Idea

For each card:

    Card → Rule Engine → Set<Role>

A card may have multiple roles.

The engine uses **deterministic pattern rules** (regex + type hints).

------------------------------------------------------------------------

## Roles To Detect

Return a `Set<Role>` from:

    LAND
    RAMP
    DRAW
    REMOVAL
    TUTOR
    PROTECTION
    FIXING
    ENGINE
    WINCON
    STAX

------------------------------------------------------------------------

## Step 1 --- Preprocessing

Normalize oracle text before matching:

-   Lowercase
-   Remove reminder text in parentheses
-   Replace newlines with spaces
-   Collapse double spaces

```{=html}
<!-- -->
```
    normalized_text = normalize(oracle_text)

------------------------------------------------------------------------

## Step 2 --- Type Hints (fast path)

If `type_line` contains:

  Type contains       Add Role
  ------------------- --------------------------------------
  Land                LAND
  Instant / Sorcery   candidate for DRAW / REMOVAL / TUTOR
  Artifact            candidate for RAMP
  Enchantment         candidate for ENGINE / STAX

Do not stop here --- continue to text rules.

------------------------------------------------------------------------

## Step 3 --- Regex Rule Table (core of system)

Apply ALL matching rules. Do not early-return.

### RAMP

Match if text contains any:

    search your library for a land
    put a land card from your hand onto the battlefield
    add {
    add one mana of any color
    create a treasure
    untap target land
    untap target artifact

------------------------------------------------------------------------

### DRAW

    draw a card
    draw two cards
    draw three cards
    whenever you draw
    investigate
    connive
    explore

------------------------------------------------------------------------

### REMOVAL

    destroy target
    exile target
    counter target spell
    return target .* to its owner's hand
    sacrifice target

------------------------------------------------------------------------

### TUTOR

    search your library for a card
    search your library for an artifact
    search your library for an instant
    search your library for an enchantment
    search your library for a creature

------------------------------------------------------------------------

### PROTECTION

    hexproof
    indestructible
    phase out
    protection from
    can't be countered

------------------------------------------------------------------------

### FIXING

    add one mana of any color
    add two mana of any one color
    any color

------------------------------------------------------------------------
## 🔶 STAX Detection Guard Clause (CRITICAL)

A card may ONLY be classified as **STAX** if the tax or restriction:

- Applies to **opponents** or **all players**, **AND**
- Affects casting spells, attacking, drawing, untapping, or activating abilities

### The following MUST NOT be classified as STAX

- Costs paid by the controller to activate or untap the card
- Mana costs that are part of the card’s own ability
- Upkeep costs, untap costs, or activation costs
- “You may pay {X}” clauses referring to the controller

### Code Logic Requirement

If the phrase contains **"you may pay"**, it must be **ignored for STAX detection**.

### Valid STAX Match Patterns

Only classify as STAX when text matches patterns such as:

- "Unless an opponent pays"
- "Spells cost {X} more to cast"
- "Players can't"
- "Each opponent must pay"
- "Opponents can't"

### STAX

    can't cast more than
    enters the battlefield tapped
    spells cost .* more to cast
    players can't
    skip their
### Additional Rule (IMPORTANT)

Any card that instructs a player to:

"Pay {mana}"  
"Pay {mana value}"  
"Pay {cost}"

as a requirement to perform a normal game action is ALSO classified as **Stax**.

This includes phrasing such as:
- "Unless they pay {1}"
- "Pay {2} for each..."
- "Spells cost {1} more..."
- "Players must pay..."

------------------------------------------------------------------------

### ENGINE (value over time)

    whenever you
    at the beginning of
    each upkeep
    each end step

------------------------------------------------------------------------

### WINCON

    you win the game
    target player loses the game
    infinite

------------------------------------------------------------------------

## Step 4 --- Multi-role is correct

Do NOT try to reduce to one label.

Examples:

-   Card can be `{RAMP, FIXING}`
-   Card can be `{DRAW, ENGINE}`
-   Card can be `{REMOVAL, STAX}`

Return all.

------------------------------------------------------------------------

## Step 5 --- Function Signature

Example (language-agnostic):

    Set<Role> infer_roles(Card card)

Where `Card` contains Scryfall fields.

------------------------------------------------------------------------

## Step 6 --- Optional Cache Layer

Create a small local table:

    card_name → inferred_roles

Only compute once per unique card.

------------------------------------------------------------------------

## Step 7 --- Validation Tests (required)

Test engine against known cards to confirm correctness:

  Card                  Expected Roles
  --------------------- ----------------
  Sol Ring              RAMP
  Cultivate             RAMP, FIXING
  Counterspell          REMOVAL
  Rhystic Study         DRAW, ENGINE
  Teferi's Protection   PROTECTION
  Demonic Tutor         TUTOR

If these fail, rules are wrong.

------------------------------------------------------------------------

## Step 8 --- Why This Works

WotC uses highly templated English in oracle text.\
Regex rules capture the majority of functional intent without tagging.

85--90% accuracy is sufficient for Monte Carlo simulation.

------------------------------------------------------------------------

## Output Example

    Sol Ring → {RAMP}
    Cultivate → {RAMP, FIXING}
    Swords to Plowshares → {REMOVAL}
    Smothering Tithe → {RAMP, ENGINE}

------------------------------------------------------------------------

## Non-Goals

-   No AI required
-   No manual tagging
-   No scanning entire Scryfall database
-   No attempt at perfect semantic understanding

------------------------------------------------------------------------

## End Result

A fast, deterministic, zero-maintenance role engine that enables:

-   Monte Carlo simulation
-   Deck composition analysis
-   Power scoring
-   AI deck suggestions
