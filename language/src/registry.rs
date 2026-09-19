use crate::{Definition, Detection, Evidence, backend::{self, Engine}, shebang};
use std::{collections::HashMap, path::Path, sync::OnceLock};

/// Validated, immutable-at-use registry. Filenames win over compound suffixes,
/// suffixes over shebangs. Upstream detection cannot perform hidden file I/O.
#[derive(Default)]
pub struct Registry {
    definitions: Vec<(Definition, Engine)>,
    names: HashMap<String, usize>,
    suffixes: HashMap<String, usize>,
    interpreters: HashMap<String, usize>,
}
impl Registry {
    pub fn builtin() -> &'static Self {
        static VALUE: OnceLock<Registry> = OnceLock::new();
        VALUE.get_or_init(|| Self::from_json(include_str!("../definitions/builtin.json"))
            .expect("built-in catalogue must pass validation tests"))
    }
    pub fn from_json(json: &str) -> Result<Self, String> {
        let definitions: Vec<Definition> = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut registry = Self::default();
        for definition in definitions { registry.register(definition)?; }
        Ok(registry)
    }
    pub fn definitions(&self) -> impl Iterator<Item=&Definition> {
        self.definitions.iter().map(|(definition, _)| definition)
    }
    /// Failed registrations never partly mutate the active catalogue.
    pub fn register(&mut self, definition: Definition) -> Result<(), String> {
        if definition.id.trim().is_empty() || definition.name.trim().is_empty() {
            return Err("Language id and display name must not be empty".into());
        }
        if self.definitions.iter().any(|(d, _)| d.id == definition.id) {
            return Err(format!("Duplicate language id: {}", definition.id));
        }
        let engine = Engine::named(&definition.classifier)?;
        let extensions: Vec<String> = definition.extensions.iter().map(|s| s.to_ascii_lowercase()).collect();
        for (rules, index) in [(&extensions, &self.suffixes), (&definition.filenames, &self.names), (&definition.interpreters, &self.interpreters)] {
            let mut seen = std::collections::HashSet::new();
            for rule in rules {
                if rule.is_empty() || rule.contains('/') || rule.contains('\\') || !seen.insert(rule) || index.contains_key(rule) {
                    return Err(format!("Invalid or duplicate language rule: {rule}"));
                }
            }
        }
        let id = self.definitions.len();
        for rule in extensions { self.suffixes.insert(rule, id); }
        for rule in &definition.filenames { self.names.insert(rule.clone(), id); }
        for rule in &definition.interpreters { self.interpreters.insert(rule.clone(), id); }
        self.definitions.push((definition, engine));
        Ok(())
    }
    fn found(&self, id: usize, evidence: Evidence) -> Detection {
        let (definition, engine) = &self.definitions[id];
        Detection { id: definition.id.clone(), name: definition.name.clone(), evidence, engine: *engine }
    }
    pub fn detect(&self, path: &Path, text: &str) -> Detection {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if let Some(&id) = self.names.get(filename) { return self.found(id, Evidence::Filename); }
        let lower = filename.to_ascii_lowercase();
        for (i, _) in lower.match_indices('.') {
            if let Some(&id) = self.suffixes.get(&lower[i+1..]) { return self.found(id, Evidence::Extension); }
        }
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if let Some(lang) = backend::extension(extension) {
            return Detection { id: lang.name().to_ascii_lowercase(), name: lang.name().into(), evidence: Evidence::Extension, engine: Engine::Tokei(lang) };
        }
        if let Some(interpreter) = shebang::interpreter(text) {
            if let Some(&id) = self.interpreters.get(interpreter) { return self.found(id, Evidence::Shebang); }
            let base = interpreter.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
            if let Some(&id) = self.interpreters.get(base) { return self.found(id, Evidence::Shebang); }
        }
        Detection { id: "unknown".into(), name: if extension.is_empty() {"Text / unknown".into()} else {format!("Unknown (.{extension})")}, evidence: Evidence::Unknown, engine: Engine::Plain }
    }
}
