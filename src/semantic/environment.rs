use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolInfo {
    name: String,
    is_initialized: bool,
    is_used: bool,
}

pub type EnvRef = Rc<RefCell<Environment>>;

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

    pub fn define_variable(env: &EnvRef, name: String, is_initialized: bool) -> bool {
        let mut env = env.borrow_mut();
        if env.variables.contains_key(&name) {
            return false;
        }
        env.variables
            .insert(name.clone(), SymbolInfo { name, is_initialized , is_used: false });
        true
    }

    pub fn is_variable_defined(env: &EnvRef, name: &str) -> bool {
        let mut current = Some(Rc::clone(env));

        while let Some(scope) = current {
            let scope_ref = scope.borrow();
            if scope_ref.variables.contains_key(name) {
                return true;
            }

            current = scope_ref.parent.as_ref().and_then(Weak::upgrade);
        }

        false
    }

    pub fn is_variable_initialized(env: &EnvRef, name: &str) -> bool {
        let mut current = Some(Rc::clone(env));

        while let Some(scope) = current {
            let scope_ref = scope.borrow();

            if let Some(info) = scope_ref.variables.get(name) {
                return info.is_initialized;
            }
            
            current = scope_ref.parent.as_ref().and_then(Weak::upgrade);
        }

        false
    }

    pub fn set_initialized(env: &EnvRef, name: &str) -> bool {
        let mut current = Some(Rc::clone(env));

        while let Some(scope) = current {
            let mut scope_ref = scope.borrow_mut();
            if let Some(symbol) = scope_ref.variables.get_mut(name) {
                symbol.is_initialized = true;
                return true;
            }

            current = scope_ref.parent.as_ref().and_then(Weak::upgrade);
        }

        false
    }

    pub fn set_used(env: &EnvRef, name: &str) -> bool {
        let mut current = Some(Rc::clone(env));

        while let Some(scope) = current {
            let mut scope_ref = scope.borrow_mut();
            if let Some(symbol) = scope_ref.variables.get_mut(name) {
                symbol.is_used = true;
                return true;
            }

            current = scope_ref.parent.as_ref().and_then(Weak::upgrade);
        }

        false
    }

    pub fn for_each_local_variable(env: &EnvRef, mut f: impl FnMut(&str, bool)) {
        let borrowed = env.borrow();
        for v in borrowed.variables.values() {
            f(&v.name, v.is_used);
        }
    }
}