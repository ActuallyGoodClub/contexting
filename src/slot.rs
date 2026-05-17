use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static SLOT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\{(\w+)\}\}").unwrap());

pub fn fill_slot(template: &str, slot: &str, content: &str) -> String {
    let escaped = regex::escape(slot);
    let pattern = format!(r"\{{\{{{}}}\}}", escaped);
    let re = Regex::new(&pattern).expect("slot regex is valid");
    re.replace_all(template, content).into_owned()
}

pub fn extract_slots(template: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for cap in SLOT_PATTERN.captures_iter(template) {
        let name = cap[1].to_string();
        if seen.insert(name.clone()) {
            result.push(name);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_slot_single() {
        assert_eq!(fill_slot("Hello {{name}}!", "name", "Alice"), "Hello Alice!");
    }

    #[test]
    fn fill_slot_multiple_occurrences() {
        assert_eq!(
            fill_slot("{{x}} and {{x}}", "x", "y"),
            "y and y"
        );
    }

    #[test]
    fn fill_slot_no_match() {
        assert_eq!(fill_slot("Hello world", "name", "Alice"), "Hello world");
    }

    #[test]
    fn extract_slots_order_and_dedup() {
        let slots = extract_slots("{{a}} {{b}} {{a}} {{c}}");
        assert_eq!(slots, vec!["a", "b", "c"]);
    }

    #[test]
    fn extract_slots_empty() {
        assert!(extract_slots("no slots here").is_empty());
    }
}
