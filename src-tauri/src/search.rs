use pinyin::{to_pinyin_vec, Pinyin};

/// Convert Chinese text to pinyin string (without tones)
pub fn to_pinyin_string(text: &str) -> String {
    to_pinyin_vec(text, |p| p.plain()).join("")
}

/// Get pinyin first letters of Chinese text
pub fn to_pinyin_initials(text: &str) -> String {
    to_pinyin_vec(text, |p: Pinyin| p.first_letter()).join("")
}

/// Fuzzy match: check if query chars appear in text in order (allowing skips)
/// e.g., "taui" matches "tauri" because t-a-u-i all appear in sequence
pub fn fuzzy_match(text: &str, query: &str) -> bool {
    let text_chars: Vec<char> = text.to_lowercase().chars().collect();
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();

    if query_chars.is_empty() {
        return true;
    }
    if text_chars.is_empty() {
        return false;
    }

    let mut query_idx = 0;
    for text_char in text_chars {
        if text_char == query_chars[query_idx] {
            query_idx += 1;
            if query_idx >= query_chars.len() {
                return true;
            }
        }
    }
    false
}

/// Calculate fuzzy match score (higher = better match, fewer skips = better)
pub fn fuzzy_match_score(text: &str, query: &str) -> i32 {
    let text_chars: Vec<char> = text.to_lowercase().chars().collect();
    let query_chars: Vec<char> = query.to_lowercase().chars().collect();

    if query_chars.is_empty() {
        return 0;
    }

    let mut query_idx = 0;
    let mut matched_positions: Vec<usize> = Vec::new();

    for (i, text_char) in text_chars.iter().enumerate() {
        if query_idx < query_chars.len() && *text_char == query_chars[query_idx] {
            matched_positions.push(i);
            query_idx += 1;
        }
    }

    if query_idx < query_chars.len() {
        return 0; // Not all query chars matched
    }

    // Score based on match quality:
    // - More matched chars = better
    // - Earlier first match = better
    // - Fewer gaps between matches = better
    let matched_count = matched_positions.len() as i32;
    let first_pos = *matched_positions.first().unwrap_or(&0) as i32;
    let gaps = if matched_positions.len() > 1 {
        matched_positions.windows(2).map(|w| (w[1] - w[0]) as i32 - 1).sum::<i32>()
    } else {
        0
    };

    // Base score for matching all chars, minus penalty for gaps and late start
    40 + matched_count * 5 - gaps * 2 - first_pos.min(10)
}

/// Check if query matches text (supports pinyin and fuzzy match)
pub fn matches(text: &str, query: &str) -> bool {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Direct substring match
    if text_lower.contains(&query_lower) {
        return true;
    }

    // Fuzzy match for non-Chinese query
    if !contains_chinese(query) {
        if fuzzy_match(text, query) {
            return true;
        }
    }

    // Pinyin full match
    let pinyin_full = to_pinyin_string(text);
    if pinyin_full.to_lowercase().contains(&query_lower) {
        return true;
    }

    // Pinyin initials match
    let pinyin_initials = to_pinyin_initials(text);
    if pinyin_initials.to_lowercase().contains(&query_lower) {
        return true;
    }

    false
}

/// Check if string contains Chinese characters
fn contains_chinese(s: &str) -> bool {
    s.chars().any(|c| c >= '\u{4E00}' && c <= '\u{9FFF}')
}

/// Calculate match score (higher = better match)
pub fn match_score(text: &str, query: &str) -> i32 {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Direct prefix match - highest score
    if text_lower.starts_with(&query_lower) {
        return 100 + query.len() as i32;
    }

    // Direct substring match
    if text_lower.contains(&query_lower) {
        return 80 + query.len() as i32;
    }

    // Fuzzy match (for non-Chinese query)
    if !contains_chinese(query) {
        let fuzzy_score = fuzzy_match_score(text, query);
        if fuzzy_score > 0 {
            return fuzzy_score;
        }
    }

    // Pinyin full prefix match
    let pinyin_full = to_pinyin_string(text);
    if pinyin_full.to_lowercase().starts_with(&query_lower) {
        return 60 + query.len() as i32;
    }

    // Pinyin full substring match
    if pinyin_full.to_lowercase().contains(&query_lower) {
        return 50 + query.len() as i32;
    }

    // Pinyin initials match
    let pinyin_initials = to_pinyin_initials(text);
    if pinyin_initials.to_lowercase().contains(&query_lower) {
        return 30 + query.len() as i32;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pinyin_string() {
        assert_eq!(to_pinyin_string("微信"), "weixin");
        assert_eq!(to_pinyin_string("Chrome"), "");
        assert_eq!(to_pinyin_string("微信Chrome"), "weixin");
    }

    #[test]
    fn test_to_pinyin_initials() {
        assert_eq!(to_pinyin_initials("微信"), "wx");
        assert_eq!(to_pinyin_initials("浏览器"), "llq");
    }

    #[test]
    fn test_matches_direct() {
        assert!(matches("Chrome", "chr"));
        assert!(matches("Visual Studio", "studio"));
    }

    #[test]
    fn test_matches_pinyin_full() {
        assert!(matches("微信", "weixin"));
        assert!(matches("微信", "wei"));
    }

    #[test]
    fn test_matches_pinyin_initials() {
        assert!(matches("微信", "wx"));
        assert!(matches("浏览器", "llq"));
    }

    #[test]
    fn test_matches_mixed() {
        assert!(matches("微信开发者工具", "wx"));
        assert!(matches("微信开发者工具", "kf"));
    }

    #[test]
    fn test_no_match() {
        assert!(!matches("Chrome", "abc"));
        assert!(!matches("微信", "xyz"));
    }

    #[test]
    fn test_match_score_direct_prefix() {
        assert!(match_score("Chrome", "chr") > match_score("Chrome", "ome"));
    }

    #[test]
    fn test_match_score_pinyin() {
        assert!(match_score("微信", "weixin") > match_score("微信", "wx"));
    }

    // Fuzzy match tests
    #[test]
    fn test_fuzzy_match_basic() {
        assert!(fuzzy_match("tauri", "taui"));
        assert!(fuzzy_match("chrome", "chre"));
        assert!(fuzzy_match("visual studio", "vstu"));
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        assert!(!fuzzy_match("tauri", "tuxi")); // 'x' not in tauri
        assert!(!fuzzy_match("chrome", "chrz")); // 'z' not in chrome
    }

    #[test]
    fn test_matches_fuzzy() {
        assert!(matches("Tauri App", "taui"));
        assert!(matches("Chrome Browser", "chre"));
    }

    #[test]
    fn test_fuzzy_match_score() {
        // Exact sequence should score higher than fuzzy
        assert!(match_score("tauri", "tauri") > match_score("tauri", "taui"));
        // Fewer gaps = higher score
        assert!(fuzzy_match_score("tauri", "taui") > fuzzy_match_score("tauri", "ti"));
    }

    #[test]
    fn test_contains_chinese() {
        assert!(contains_chinese("微信"));
        assert!(contains_chinese("abc微信def"));
        assert!(!contains_chinese("chrome"));
        assert!(!contains_chinese(""));
    }
}