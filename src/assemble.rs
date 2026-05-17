use crate::slot::{extract_slots, fill_slot};
use crate::types::{Context, Injector};
use serde_json::Value;

fn apply_injector(
    template: &str,
    injector: &Injector,
    context: &Context,
) -> (String, Option<String>) {
    let injection = (injector.fn_)(context);
    let slot_tag = format!("{{{{{}}}}}", injector.slot);

    if template.contains(&slot_tag) {
        (fill_slot(template, &injector.slot, &injection), None)
    } else if !injection.is_empty() {
        (template.to_string(), Some(injection))
    } else {
        (template.to_string(), None)
    }
}

fn apply_base_rules(template: &str, context: &Context) -> String {
    let mut current = template.to_string();
    for slot in extract_slots(template) {
        if let Some(value) = context.get(&slot) {
            let replacement = match value {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            };
            if let Some(text) = replacement {
                current = fill_slot(&current, &slot, &text);
            }
        }
    }
    current
}

pub fn assemble(base_prompt: &str, rules: &[Injector], context: &Context) -> String {
    let mut current_template = base_prompt.to_string();
    let mut trailing_parts: Vec<String> = Vec::new();

    for rule in rules {
        let (new_template, appended) = apply_injector(&current_template, rule, context);
        current_template = new_template;
        if let Some(text) = appended {
            trailing_parts.push(text);
        }
    }

    let filled = apply_base_rules(&current_template, context);

    if trailing_parts.is_empty() {
        filled
    } else {
        format!("{}\n\n{}", filled, trailing_parts.join("\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::injector::create_injector;
    use std::collections::HashMap;

    fn ctx(pairs: &[(&str, &str)]) -> Context {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
            .collect()
    }

    #[test]
    fn inline_injection() {
        let rules = vec![
            create_injector("role", Box::new(|_| "senior engineer".to_string())).unwrap(),
        ];
        let result = assemble("You are a {{role}}.", &rules, &ctx(&[]));
        assert_eq!(result, "You are a senior engineer.");
    }

    #[test]
    fn appended_injection() {
        let rules = vec![
            create_injector("rules", Box::new(|_| "Be safe.".to_string())).unwrap(),
        ];
        let result = assemble("Hello {{name}}.", &rules, &ctx(&[("name", "Alice")]));
        assert_eq!(result, "Hello Alice.\n\nBe safe.");
    }

    #[test]
    fn empty_appended_not_added() {
        let rules = vec![
            create_injector("rules", Box::new(|_| String::new())).unwrap(),
        ];
        let result = assemble("Hello.", &rules, &ctx(&[]));
        assert_eq!(result, "Hello.");
    }

    #[test]
    fn base_rule_fill_from_context() {
        let result = assemble("Hi {{name}}!", &[], &ctx(&[("name", "Bob")]));
        assert_eq!(result, "Hi Bob!");
    }

    #[test]
    fn multiple_injectors_and_trailing() {
        let rules = vec![
            create_injector("role", Box::new(|_| "assistant".to_string())).unwrap(),
            create_injector("extra", Box::new(|_| "Be concise.".to_string())).unwrap(),
            create_injector("more", Box::new(|_| "Be helpful.".to_string())).unwrap(),
        ];
        let result = assemble("You are a {{role}}.", &rules, &ctx(&[]));
        assert_eq!(result, "You are a assistant.\n\nBe concise.\n\nBe helpful.");
    }

    #[test]
    fn context_dependent_injection() {
        let rules = vec![
            create_injector(
                "role",
                Box::new(|ctx| {
                    if ctx.get("senior") == Some(&Value::Bool(true)) {
                        "senior".to_string()
                    } else {
                        "junior".to_string()
                    }
                }),
            )
            .unwrap(),
        ];
        let mut context: Context = HashMap::new();
        context.insert("senior".to_string(), Value::Bool(true));
        let result = assemble("You are a {{role}}.", &rules, &context);
        assert_eq!(result, "You are a senior.");
    }
}
