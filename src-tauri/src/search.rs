use pinyin::{to_pinyin_vec, Pinyin};

/// Convert Chinese text to pinyin string (without tones)
pub fn to_pinyin_string(text: &str) -> String {
    to_pinyin_vec(text, |p| p.plain()).join("")
}

/// Get pinyin first letters of Chinese text
pub fn to_pinyin_initials(text: &str) -> String {
    to_pinyin_vec(text, |p: Pinyin| p.first_letter()).join("")
}

/// Check if query matches text (supports pinyin)
pub fn matches(text: &str, query: &str) -> bool {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Direct substring match
    if text_lower.contains(&query_lower) {
        return true;
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
}