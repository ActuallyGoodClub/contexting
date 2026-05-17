use crate::types::{Injector, InjectorFn};

#[derive(Debug, thiserror::Error)]
pub enum InjectorError {
    #[error("slot must be a non-empty string")]
    EmptySlot,
}

pub fn create_injector(
    slot: impl Into<String>,
    fn_: InjectorFn,
) -> Result<Injector, InjectorError> {
    let slot = slot.into();
    if slot.trim().is_empty() {
        return Err(InjectorError::EmptySlot);
    }
    Ok(Injector { slot, fn_ })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_injector_valid() {
        let inj = create_injector("role", Box::new(|_| "engineer".to_string()));
        assert!(inj.is_ok());
        assert_eq!(inj.unwrap().slot, "role");
    }

    #[test]
    fn create_injector_empty_slot() {
        assert!(create_injector("", Box::new(|_| "x".to_string())).is_err());
        assert!(create_injector("  ", Box::new(|_| "x".to_string())).is_err());
    }
}
