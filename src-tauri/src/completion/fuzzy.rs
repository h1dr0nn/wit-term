//! Fuzzy matching algorithm for completions.

/// Calculate a fuzzy match score between a query and a candidate.
/// Returns None if there's no match, or Some(score) where higher is better.
pub fn fuzzy_match(query: &str, candidate: &str) -> Option<f64> {
    if query.is_empty() {
        return Some(0.5); // Empty query matches everything with low score
    }

    let query_lower = query.to_lowercase();
    let candidate_lower = candidate.to_lowercase();

    // Exact prefix match gets highest score
    if candidate_lower.starts_with(&query_lower) {
        let ratio = query.len() as f64 / candidate.len() as f64;
        return Some(1.0 + ratio);
    }

    // Substring match
    if candidate_lower.contains(&query_lower) {
        let ratio = query.len() as f64 / candidate.len() as f64;
        return Some(0.7 + ratio * 0.3);
    }

    // Fuzzy subsequence match
    let mut query_chars = query_lower.chars().peekable();
    let mut matched = 0;
    let mut consecutive = 0;
    let mut max_consecutive = 0;
    let mut last_match_pos: Option<usize> = None;
    let mut gap_penalty = 0.0;

    for (i, ch) in candidate_lower.chars().enumerate() {
        if let Some(&qch) = query_chars.peek() {
            if ch == qch {
                query_chars.next();
                matched += 1;

                if last_match_pos.is_some_and(|p| p == i - 1) {
                    consecutive += 1;
                    max_consecutive = max_consecutive.max(consecutive);
                } else {
                    consecutive = 1;
                    if let Some(prev) = last_match_pos {
                        gap_penalty += (i - prev - 1) as f64 * 0.05;
                    }
                }
                last_match_pos = Some(i);
            }
        }
    }

    // All query characters must match
    if query_chars.peek().is_some() {
        return None;
    }

    let base_score = matched as f64 / candidate.len() as f64;
    let consecutive_bonus = max_consecutive as f64 * 0.1;
    let score = (base_score + consecutive_bonus - gap_penalty).max(0.01);

    Some(score.min(0.99)) // Cap below exact match scores
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_prefix() {
        let score = fuzzy_match("che", "checkout").unwrap();
        assert!(score > 1.0, "Prefix match should score > 1.0: {score}");
    }

    #[test]
    fn test_substring() {
        let score = fuzzy_match("out", "checkout").unwrap();
        assert!(score > 0.5, "Substring should score > 0.5: {score}");
    }

    #[test]
    fn test_fuzzy_subsequence() {
        let score = fuzzy_match("chk", "checkout").unwrap();
        assert!(score > 0.0, "Fuzzy match should have positive score: {score}");
    }

    #[test]
    fn test_no_match() {
        assert!(fuzzy_match("xyz", "checkout").is_none());
    }

    #[test]
    fn test_empty_query() {
        let score = fuzzy_match("", "checkout").unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn test_case_insensitive() {
        let score = fuzzy_match("CHE", "checkout").unwrap();
        assert!(score > 1.0);
    }

    #[test]
    fn test_prefix_beats_fuzzy() {
        let prefix = fuzzy_match("co", "commit").unwrap();
        let fuzzy = fuzzy_match("ct", "commit").unwrap();
        assert!(prefix > fuzzy);
    }

    #[test]
    fn test_exact_match() {
        let score = fuzzy_match("git", "git").unwrap();
        assert!(score > 1.5, "Exact match should score high: {score}");
    }
}
