# CRISPI Extension: Archetype Coherence + Win Density

This document describes **exactly** how to extend the CRISPI model so that decks with many strong roles but no coherent competitive gameplan are prevented from scoring as Fringe cEDH or cEDH.

The goal is to stop role score inflation from interaction, protection, and draw, and instead reward:

- Focused archetype alignment
- Deterministic win line density
- True cEDH shell structure

This extension MUST be applied AFTER core CRISPI (C/R/I/S/P) is calculated and BEFORE the final score multiplier is applied.

---

## Step 1 — Calculate Archetype Signals (already exists)

These values already exist in the system:

- `turbo_signal`
- `midrange_signal`
- `stax_signal`
- `commander_engine_signal`

Do NOT change how these are calculated.

---

## Step 2 — Compute Signal Focus and Magnitude

Add this logic after signals are computed.

```rust
let signals = [
    turbo_signal,
    midrange_signal,
    stax_signal,
    commander_engine_signal,
];

let max_signal = signals.iter().cloned().fold(0./0., f32::max);
let sum_signal: f32 = signals.iter().sum();

let focus_ratio = if sum_signal > 0.0 {
    max_signal / sum_signal
} else { 0.0 };

let magnitude = max_signal;
```

### Meaning

- High focus_ratio = deck is committed to one archetype
- Low focus_ratio = pile of good cards
- High magnitude = strong archetype presence

---

## Step 3 — Archetype Coherence Multiplier

Apply this multiplier to `final_multiplier`.

```rust
let coherence_multiplier = match (focus_ratio, magnitude) {
    (r, m) if r >= 0.45 && m >= 35.0 => 1.10,
    (r, m) if r >= 0.40 && m >= 25.0 => 1.04,
    (r, m) if r >= 0.35 && m >= 18.0 => 0.98,
    (r, _) if r < 0.35 => 0.90,
    _ => 1.0,
};

final_multiplier *= coherence_multiplier;
```

---

## Step 4 — Introduce Win Line Density

Create a new role classification bucket if needed:

- `WINCON`
- `COMBO_PIECE`
- `ENGINE_WIN`
- `TUTOR`
- `RECURSION`

Then compute:

```rust
let win_pieces = count(cards where role in [WINCON, COMBO_PIECE, ENGINE_WIN]);
let tutors = count(cards where role == TUTOR);
let recursion = count(cards where role == RECURSION);

let win_density = win_pieces as f32
    + tutors as f32 * 0.7
    + recursion as f32 * 0.5;
```

---

## Step 5 — Win Gate Multiplier

Apply AFTER coherence multiplier.

```rust
let win_gate = match win_density {
    d if d >= 18.0 => 1.08,
    d if d >= 12.0 => 1.02,
    d if d >= 7.0  => 0.95,
    _              => 0.82,
};

final_multiplier *= win_gate;
```

---

## Step 6 — cEDH Staples Reference File

Create a separate Rust file:

`cedh_staples.rs`

This file contains a static HashSet of card names known to be part of real cEDH win shells.

```rust
use std::collections::HashSet;
use once_cell::sync::Lazy;

pub static CEDH_STAPLES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "Thassa's Oracle",
        "Demonic Consultation",
        "Tainted Pact",
        "Underworld Breach",
        "Ad Nauseam",
        "Dockside Extortionist",
        "Brain Freeze",
        "Lion's Eye Diamond",
        "Intuition",
        "Grinding Station",
        "Isochron Scepter",
        "Dramatic Reversal",
        "Finale of Devastation",
        "Birthing Pod",
        "Necropotence",
        "Peer into the Abyss",
        "Yawgmoth's Will",
        "Sevinne's Reclamation",
        "Wishclaw Talisman",
    ]
    .iter()
    .cloned()
    .collect()
});
```

Then add:

```rust
let staple_hits = count(cards where name in CEDH_STAPLES);
```

Optional bonus multiplier:

```rust
if staple_hits >= 6 {
    final_multiplier *= 1.05;
}
```

This should NOT carry decks, only slightly reward real shells.

---

## Final Order of Operations

1. Compute CRISPI core score
2. Compute archetype signals
3. Apply coherence multiplier
4. Compute win density
5. Apply win gate multiplier
6. Check staple hits (optional small bonus)
7. Apply final multiplier to CRISPI score

---

## Expected Behavioral Change

Decks with:

- Lots of interaction but no win shell → downgraded
- Focused archetype and real win lines → upgraded
- Goodstuff piles → penalized
- True cEDH lists → clearly separated from fringe

This fixes the misclassification problem without altering CRISPI core math.

