use contexting_core::slot::{extract_slots, fill_slot};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use serde_json::Value;
use std::collections::HashMap;

type Context = HashMap<String, Value>;

/// Python-facing Injector storing a Python callable.
#[pyclass]
struct Injector {
    slot: String,
    callable: PyObject,
}

#[pymethods]
impl Injector {
    #[new]
    fn new(slot: &str, callable: PyObject) -> PyResult<Self> {
        if slot.trim().is_empty() {
            return Err(PyValueError::new_err("slot must be a non-empty string"));
        }
        Ok(Self { slot: slot.to_string(), callable })
    }

    #[getter]
    fn slot(&self) -> &str {
        &self.slot
    }
}

fn py_dict_to_context(dict: &Bound<'_, PyDict>) -> PyResult<Context> {
    let mut map = HashMap::new();
    for (k, v) in dict.iter() {
        let key: String = k.extract()?;
        let value = py_to_json_value(&v)?;
        map.insert(key, value);
    }
    Ok(map)
}

fn py_to_json_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        Ok(Value::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(Value::Number(i.into()))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Value::Number(
            serde_json::Number::from_f64(f).unwrap_or_else(|| 0i64.into()),
        ))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value::String(s))
    } else {
        let s: String = obj.str()?.extract()?;
        Ok(Value::String(s))
    }
}

fn json_to_py(py: Python, val: &Value) -> PyObject {
    match val {
        Value::Null => py.None(),
        Value::Bool(b) => b.into_py(py),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py(py)
            } else if let Some(f) = n.as_f64() {
                f.into_py(py)
            } else {
                py.None()
            }
        }
        Value::String(s) => PyString::new_bound(py, s).into_py(py),
        _ => py.None(),
    }
}

fn context_to_py_dict<'py>(py: Python<'py>, ctx: &Context) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new_bound(py);
    for (k, v) in ctx {
        dict.set_item(k, json_to_py(py, v))?;
    }
    Ok(dict)
}

#[pyfunction]
fn create_injector(slot: &str, callable: PyObject) -> PyResult<Injector> {
    Injector::new(slot, callable)
}

#[pyfunction]
fn assemble(
    py: Python,
    base_prompt: &str,
    rules: Vec<PyRef<Injector>>,
    context: &Bound<'_, PyDict>,
) -> PyResult<String> {
    let ctx = py_dict_to_context(context)?;
    let py_ctx = context_to_py_dict(py, &ctx)?;

    let mut current_template = base_prompt.to_string();
    let mut trailing_parts: Vec<String> = Vec::new();

    for rule in &rules {
        let injection: String = rule
            .callable
            .call1(py, (&py_ctx,))?
            .extract(py)?;

        let slot_tag = format!("{{{{{}}}}}", rule.slot);
        if current_template.contains(&slot_tag) {
            current_template = fill_slot(&current_template, &rule.slot, &injection);
        } else if !injection.is_empty() {
            trailing_parts.push(injection);
        }
    }

    // Apply base rules from context
    let slots = extract_slots(&current_template);
    for slot in slots {
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

#[pymodule]
fn contexting(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Injector>()?;
    m.add_function(wrap_pyfunction!(create_injector, m)?)?;
    m.add_function(wrap_pyfunction!(assemble, m)?)?;
    Ok(())
}
