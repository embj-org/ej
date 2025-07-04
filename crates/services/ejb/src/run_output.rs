use std::collections::HashMap;

use ej_config::ej_config::EjConfig;
use uuid::Uuid;

#[derive(Debug)]
pub struct EjRunOutput<'a> {
    pub config: &'a EjConfig,
    pub logs: HashMap<Uuid, Vec<String>>,
    pub results: HashMap<Uuid, String>,
}
impl<'a> EjRunOutput<'a> {
    pub fn new(config: &'a EjConfig) -> Self {
        Self {
            config,
            logs: HashMap::new(),
            results: HashMap::new(),
        }
    }
    pub fn reset(&mut self) {
        self.logs.clear();
        self.results.clear();
    }
}
