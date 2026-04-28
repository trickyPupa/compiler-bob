use std::{cell::RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::common::data_type::DataType;
use crate::common::symbol::SymbolInfo;

pub type EnvRef = Rc<RefCell<Environment>>;

pub trait EnvOps {
    fn define_variable(&self, name: String, is_initialized: bool, dtype: DataType) -> bool;
    fn is_variable_defined(&self, name: &str) -> bool;
    fn is_variable_initialized(&self, name: &str) -> bool;
    fn set_initialized(&self, name: &str) -> bool;
    fn set_used(&self, name: &str) -> bool;
    fn for_each_local_variable(&self, f: impl FnMut(&str, bool));
    fn with_variable<R>(&self, name: &str, f: impl FnMut(&SymbolInfo) -> R) -> Option<R>;
    fn with_variable_mut<R>(&self, name: &str, f: impl FnMut(&mut SymbolInfo) -> R) -> Option<R>;
    fn variables(&self) -> HashMap<String, SymbolInfo>;
}

impl EnvOps for EnvRef {
    fn define_variable(&self, name: String, is_initialized: bool, dtype: DataType) -> bool {
        Environment::define_variable(self, name, is_initialized, dtype)
    }

    fn is_variable_defined(&self, name: &str) -> bool {
        Environment::is_variable_defined(self, name)
    }

    fn is_variable_initialized(&self, name: &str) -> bool {
        Environment::is_variable_initialized(self, name)
    }

    fn set_initialized(&self, name: &str) -> bool {
        Environment::set_initialized(self, name)
    }

    fn set_used(&self, name: &str) -> bool {
        Environment::set_used(self, name)
    }

    fn for_each_local_variable(&self, f: impl FnMut(&str, bool)) {
        Environment::for_each_local_variable(self, f)
    }

    fn with_variable<R>(&self, name: &str, f: impl FnMut(&SymbolInfo) -> R) -> Option<R> {
        Environment::with_variable(self, name, f)
    }

    fn with_variable_mut<R>(&self, name: &str, f: impl FnMut(&mut SymbolInfo) -> R) -> Option<R> {
        Environment::with_variable_mut(self, name, f)
    }

    fn variables(&self) -> HashMap<String, SymbolInfo> {
        self.borrow().variables.clone()
    }
}

pub struct Environment {
    parent: Option<Weak<RefCell<Environment>>>,
    variables: HashMap<String, SymbolInfo>,
}

impl Environment {
    pub fn new(parent: Option<&EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            parent: parent.map(Rc::downgrade),
            variables: HashMap::new(),
        }))
    }

    pub fn define_variable(
        env: &EnvRef,
        name: String,
        is_initialized: bool,
        dtype: DataType,
    ) -> bool {
        let mut env = env.borrow_mut();
        if env.variables.contains_key(&name) {
            return false;
        }
        env.variables.insert(
            name.clone(),
            SymbolInfo {
                name,
                is_initialized,
                is_used: false,
                dtype,
            },
        );
        true
    }

    pub fn is_variable_defined(env: &EnvRef, name: &str) -> bool {
        Self::with_variable(env, name, |_| ()).is_some()
    }

    pub fn is_variable_initialized(env: &EnvRef, name: &str) -> bool {
        Self::with_variable(env, name, |s| s.is_initialized).unwrap_or(false)
    }

    pub fn set_initialized(env: &EnvRef, name: &str) -> bool {
        Self::with_variable_mut(env, name, |s| s.is_initialized = true).is_some()
    }

    pub fn set_used(env: &EnvRef, name: &str) -> bool {
        Self::with_variable_mut(env, name, |s| s.is_used = true).is_some()
    }

    pub fn for_each_local_variable(env: &EnvRef, mut f: impl FnMut(&str, bool)) {
        let borrowed = env.borrow();
        for v in borrowed.variables.values() {
            f(&v.name, v.is_used);
        }
    }

    pub fn with_variable<R>(
        env: &EnvRef,
        name: &str,
        mut f: impl FnMut(&SymbolInfo) -> R,
    ) -> Option<R> {
        let mut current = Some(Rc::clone(env));

        while let Some(scope) = current {
            let next = {
                let borrowed = scope.borrow();

                if let Some(sym) = borrowed.variables.get(name) {
                    return Some(f(sym));
                }

                borrowed.parent.as_ref().and_then(Weak::upgrade)
            };

            current = next;
        }

        None
    }

    pub fn with_variable_mut<R>(
        env: &EnvRef,
        name: &str,
        mut f: impl FnMut(&mut SymbolInfo) -> R,
    ) -> Option<R> {
        let mut current = Some(Rc::clone(env));

        while let Some(scope) = current {
            let next = {
                let mut borrowed = scope.borrow_mut();

                if let Some(sym) = borrowed.variables.get_mut(name) {
                    return Some(f(sym));
                }

                borrowed.parent.as_ref().and_then(Weak::upgrade)
            };

            current = next;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{EnvOps, EnvRef, Environment};
    use crate::common::data_type::DataType;

    fn child_of(parent: &EnvRef) -> EnvRef {
        Environment::new(Some(parent))
    }

    #[test]
    fn define_and_find_variable_in_current_scope() {
        let env = Environment::new(None);

        assert!(env.define_variable("x".to_string(), false, DataType::Numeric));
        assert!(env.is_variable_defined("x"));
        assert!(!env.is_variable_initialized("x"));
    }

    #[test]
    fn duplicate_variable_in_same_scope_is_rejected() {
        let env = Environment::new(None);

        assert!(env.define_variable("x".to_string(), true, DataType::Numeric));
        assert!(!env.define_variable("x".to_string(), false, DataType::Boolean));
    }

    #[test]
    fn child_scope_can_see_parent_variable() {
        let parent = Environment::new(None);
        assert!(parent.define_variable("x".to_string(), true, DataType::Numeric));

        let child = child_of(&parent);
        assert!(child.is_variable_defined("x"));
        assert!(child.is_variable_initialized("x"));
    }

    #[test]
    fn set_initialized_updates_nearest_scope_entry() {
        let parent = Environment::new(None);
        assert!(parent.define_variable("x".to_string(), false, DataType::Numeric));

        let child = child_of(&parent);
        assert!(child.set_initialized("x"));
        assert!(parent.is_variable_initialized("x"));
    }

    #[test]
    fn set_used_marks_symbol_as_used() {
        let env = Environment::new(None);
        assert!(env.define_variable("x".to_string(), true, DataType::Numeric));

        assert!(env.set_used("x"));
        let is_used = env.with_variable("x", |s| s.is_used).unwrap_or(false);
        assert!(is_used);
    }

    #[test]
    fn for_each_local_variable_iterates_only_local_scope() {
        let parent = Environment::new(None);
        assert!(parent.define_variable("parent_only".to_string(), true, DataType::Numeric));

        let child = child_of(&parent);
        assert!(child.define_variable("child_only".to_string(), false, DataType::Boolean));

        let mut names = Vec::new();
        child.for_each_local_variable(|name, _| names.push(name.to_string()));

        assert_eq!(names, vec!["child_only".to_string()]);
    }

    #[test]
    fn with_variable_and_with_variable_mut_work_through_parent_chain() {
        let parent = Environment::new(None);
        assert!(parent.define_variable("x".to_string(), false, DataType::Numeric));
        let child = child_of(&parent);

        let was_initialized = child.with_variable("x", |s| s.is_initialized);
        assert_eq!(was_initialized, Some(false));

        let mark = child.with_variable_mut("x", |s| {
            s.is_initialized = true;
            s.is_used = true;
        });
        assert!(mark.is_some());

        let state = parent.with_variable("x", |s| (s.is_initialized, s.is_used));
        assert_eq!(state, Some((true, true)));
    }
}
