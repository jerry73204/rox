use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Debug)]
pub struct VarStore {
    scopes: Vec<Scope>,
    vars: HashMap<Arc<str>, Vec<String>>,
    envs: HashMap<Arc<str>, Vec<String>>,
}

#[derive(Debug)]
struct Scope {
    var_keys: HashSet<Arc<str>>,
    env_keys: HashSet<Arc<str>>,
}

impl Default for VarStore {
    fn default() -> Self {
        Self {
            scopes: vec![],
            vars: HashMap::new(),
            envs: HashMap::new(),
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            var_keys: HashSet::new(),
            env_keys: HashSet::new(),
        }
    }
}

impl VarStore {
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) {
        let scope = self.scopes.pop().unwrap();
        for key in scope.var_keys {
            self.vars.get_mut(&key).unwrap().pop().unwrap();
        }
        for key in scope.env_keys {
            self.envs.get_mut(&key).unwrap().pop().unwrap();
        }
    }

    pub fn contains_var(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    pub fn contains_env(&self, name: &str) -> bool {
        self.envs.contains_key(name)
    }

    pub fn insert_var(&mut self, key: String, value: String) -> &str {
        let key: Arc<str> = key.into_boxed_str().into();
        self.current_scope_mut().var_keys.insert(key.clone());
        let entry = self.vars.entry(key).or_default();
        entry.push(value);
        entry.last().unwrap()
    }

    pub fn insert_env(&mut self, key: String, value: String) -> &str {
        let key: Arc<str> = key.into_boxed_str().into();
        self.current_scope_mut().env_keys.insert(key.clone());
        let entry = self.envs.entry(key).or_default();
        entry.push(value);
        entry.last().unwrap()
    }

    pub fn get_var(&self, key: &str) -> Option<&str> {
        self.vars
            .get(key)
            .and_then(|v| v.last())
            .map(|s| s.as_str())
    }

    pub fn get_var_or_insert(&mut self, key: &str, default: &str) -> String {
        if let Some(value) = self.get_var(key) {
            value.to_string()
        } else {
            self.insert_var(key.to_string(), default.to_string())
                .to_string()
        }
    }

    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.envs
            .get(key)
            .and_then(|v| v.last())
            .map(|s| s.as_str())
    }

    pub fn get_env_or_insert(&mut self, key: &str, default: &str) -> String {
        if let Some(value) = self.get_env(key) {
            value.to_string()
        } else {
            self.insert_env(key.to_string(), default.to_string())
                .to_string()
        }
    }

    pub fn remove_var(&mut self, key: &str) -> Option<String> {
        if self.current_scope_mut().var_keys.remove(key) {
            let value = self.vars.get_mut(key).unwrap().pop().unwrap();
            Some(value)
        } else {
            None
        }
    }

    pub fn remove_env(&mut self, key: &str) -> Option<String> {
        if self.current_scope_mut().env_keys.remove(key) {
            let value = self.envs.get_mut(key).unwrap().pop().unwrap();
            Some(value)
        } else {
            None
        }
    }

    fn current_scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }
}
