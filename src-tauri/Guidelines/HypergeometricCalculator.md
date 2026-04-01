
# Hypergeometric Calculator Specification

## Purpose
Provide exact probability calculations for drawing specific card types from a Commander deck using the hypergeometric distribution.

The calculator MUST report the following probabilities for a desired card category:
- Probability of drawing **none**
- Probability of drawing **exactly N**
- Probability of drawing **more than N**
- Probability of drawing **less than N**

This is used for evaluating deck consistency, tutor density, ramp density, interaction density, etc.

---

## Inputs

| Variable | Meaning |
|----------|---------|
| N | Total cards in deck (usually 99) |
| K | Total desired cards in deck (e.g., ramp pieces) |
| n | Cards drawn (opening hand, turn X, etc.) |
| k | Desired number of hits |

---

## Core Formula

Hypergeometric probability:

P(X = k) = [ C(K, k) * C(N-K, n-k) ] / C(N, n)

Where C(a, b) is the combination function.

---

## Required Outputs

For a given k, the calculator MUST compute:

1. **P(X = 0)** → probability of drawing none  
2. **P(X = k)** → probability of drawing exactly k  
3. **P(X > k)** → probability of drawing more than k  
4. **P(X < k)** → probability of drawing less than k  

### Derived Calculations

P(X > k) = 1 - Σ P(X = i) for i = 0..k  
P(X < k) = Σ P(X = i) for i = 0..(k-1)

---

## Usage Examples

- Chance to see at least 1 ramp spell by turn 3
- Chance to open with 2 interaction pieces
- Chance to miss all tutors in opening hand
- Chance to draw 3+ combo pieces by turn 6

---

## Notes for Implementation

- Use precise math (no Monte Carlo approximation)
- Support dynamic inputs for different turns and draw counts
- Designed to work with card categories inferred by the inference engine
