use std::collections::HashMap;
use serde_json::Value;

pub type Context = HashMap<String, Value>;
pub type InjectorFn = Box<dyn Fn(&Context) -> String + Send + Sync>;

pub struct Injector {
    pub slot: String,
    pub fn_: InjectorFn,
}

pub type Rules = Vec<Injector>;
