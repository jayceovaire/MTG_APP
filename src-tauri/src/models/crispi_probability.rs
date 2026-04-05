use crate::models::card_model::Card;

pub fn combinations(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    if k == 0 || k == n {
        return 1.0;
    }

    let k = k.min(n - k);
    let mut result = 1.0;
    for i in 1..=k {
        result *= (n - k + i) as f64 / i as f64;
    }
    result
}

pub fn hypergeometric_at_least(population: usize, successes: usize, draws: usize, target: usize) -> f32 {
    if population == 0 || draws == 0 || successes == 0 || target > draws {
        return 0.0;
    }

    let draws = draws.min(population);
    let max_hits = successes.min(draws);
    if target > max_hits {
        return 0.0;
    }

    let denominator = combinations(population, draws);
    if denominator == 0.0 {
        return 0.0;
    }

    let mut total = 0.0;
    for hits in target..=max_hits {
        total += combinations(successes, hits) * combinations(population - successes, draws - hits) / denominator;
    }
    total as f32
}

pub fn derive_bracket(n_gc: u32, any_combo_found: bool, total_score: f32, amv: f32) -> u8 {
    let mut bracket = if n_gc == 0 {
        2
    } else if n_gc <= 3 {
        3
    } else {
        4
    };

    if any_combo_found {
        bracket = bracket.max(4);
    }

    if bracket == 2 && total_score <= 8.0 && amv > 3.5 {
        1
    } else if total_score >= 24.0 {
        5
    } else {
        bracket
    }
}

pub fn is_etb_tapped_land<F>(card: &Card, normalize_text: F) -> bool
where
    F: Fn(&str) -> String,
{
    if !card.is_land() {
        return false;
    }

    let oracle_text = card.oracle_text().map(normalize_text).unwrap_or_default();
    oracle_text.contains("enters the battlefield tapped") && !oracle_text.contains("unless")
}
