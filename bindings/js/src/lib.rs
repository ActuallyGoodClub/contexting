use contexting_core::slot::{extract_slots, fill_slot};
use js_sys::{Function, Object, Reflect};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

type Context = HashMap<String, Value>;

/// An injector object: `{ slot: string, func: (ctx) => string }`.
/// Created by `createInjector` and passed in an array to `assemble`.
#[wasm_bindgen]
pub struct Injector {
    slot: String,
    func: Function,
}

#[wasm_bindgen]
impl Injector {
    #[wasm_bindgen(getter)]
    pub fn slot(&self) -> String {
        self.slot.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn func(&self) -> Function {
        self.func.clone()
    }
}

/// Factory matching the TypeScript `createInjector` API.
#[wasm_bindgen(js_name = createInjector)]
pub fn create_injector(slot: &str, func: Function) -> Result<Injector, JsValue> {
    if slot.trim().is_empty() {
        return Err(JsValue::from_str("slot must be a non-empty string"));
    }
    Ok(Injector { slot: slot.to_string(), func })
}

/// Assemble a prompt from a template, an array of injectors, and a plain JS context object.
/// API matches the TypeScript original: `assemble(basePrompt, [inj1, inj2], context)`.
#[wasm_bindgen]
pub fn assemble(
    base_prompt: &str,
    rules: Box<[JsValue]>,
    context: JsValue,
) -> Result<String, JsValue> {
    let ctx = js_object_to_context(&context);
    let js_ctx = context_to_js_object(&ctx);

    let mut current_template = base_prompt.to_string();
    let mut trailing_parts: Vec<String> = Vec::new();

    for item in rules.iter() {
        let slot = Reflect::get(item, &JsValue::from_str("slot"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let func = Reflect::get(item, &JsValue::from_str("func"))
            .ok()
            .and_then(|v| v.dyn_into::<Function>().ok());

        let Some(func) = func else { continue };

        let injection = func.call1(&JsValue::null(), &js_ctx).map_err(|e| e)?;
        let injection_str = injection.as_string().unwrap_or_default();

        let slot_tag = format!("{{{{{}}}}}", slot);
        if current_template.contains(&slot_tag) {
            current_template = fill_slot(&current_template, &slot, &injection_str);
        } else if !injection_str.is_empty() {
            trailing_parts.push(injection_str);
        }
    }

    // Fill remaining slots from context
    for slot in extract_slots(&current_template) {
        if let Some(val) = ctx.get(&slot) {
            let replacement = match val {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            };
            if let Some(text) = replacement {
                current_template = fill_slot(&current_template, &slot, &text);
            }
        }
    }

    if trailing_parts.is_empty() {
        Ok(current_template)
    } else {
        Ok(format!("{}\n\n{}", current_template, trailing_parts.join("\n\n")))
    }
}

fn js_object_to_context(obj: &JsValue) -> Context {
    let mut map = HashMap::new();
    if let Some(object) = obj.dyn_ref::<Object>() {
        if let Ok(keys) = Object::keys(object).dyn_into::<js_sys::Array>() {
            for key in keys.iter() {
                if let Some(k) = key.as_string() {
                    if let Ok(val) = Reflect::get(object, &key) {
                        map.insert(k, js_value_to_json(&val));
                    }
                }
            }
        }
    }
    map
}

fn js_value_to_json(val: &JsValue) -> Value {
    if val.is_null() || val.is_undefined() {
        Value::Null
    } else if let Some(b) = val.as_bool() {
        Value::Bool(b)
    } else if let Some(f) = val.as_f64() {
        if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
            Value::Number((f as i64).into())
        } else {
            Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| 0i64.into()))
        }
    } else if let Some(s) = val.as_string() {
        Value::String(s)
    } else {
        Value::Null
    }
}

fn context_to_js_object(ctx: &Context) -> Object {
    let obj = Object::new();
    for (k, v) in ctx {
        let _ = Reflect::set(&obj, &JsValue::from_str(k), &json_to_js_value(v));
    }
    obj
}

fn json_to_js_value(val: &Value) -> JsValue {
    match val {
        Value::Null => JsValue::null(),
        Value::Bool(b) => JsValue::from_bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { JsValue::from_f64(i as f64) }
            else if let Some(f) = n.as_f64() { JsValue::from_f64(f) }
            else { JsValue::null() }
        }
        Value::String(s) => JsValue::from_str(s),
        _ => JsValue::null(),
    }
}
