use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use color_eyre::eyre::{eyre, Result};
use icon_updater::IconUpdater;
use serde::{Deserialize, Serialize};
use serenity::all::Http;

mod icon_updater;
mod util;

type State = String;

#[async_trait]
pub(crate) trait Module {
    fn name(&self) -> &'static str;
    fn load<'a>(state: Option<&str>, http: Arc<Http>) -> Result<Self>
    where
        Self: Sized;
    async fn state(&self) -> State;
    async fn shutdown(self: Box<Self>) -> State;
}

pub struct Bot {
    state: BotState,
    modules: Vec<Box<dyn Module>>,
    http: Arc<Http>,
}
impl Bot {
    pub fn new(state: &BotState, http: Http) -> Result<Bot> {
        let http = Arc::new(http);

        let modules: Vec<Box<dyn Module>> = state
            .enabled_modules
            .iter()
            .map(|module_name| match module_name.as_str() {
                icon_updater::NAME => IconUpdater::load(
                    state
                        .module_states
                        .get(module_name.as_str())
                        .map(String::as_str),
                    Arc::clone(&http),
                )
                .map(|module| Box::new(module) as Box<dyn Module>)
                .map_err(|e| eyre!("Error loading module {module_name}: {e}")),
                unknown => Err(eyre!("Unknown module name {unknown}")),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Bot {
            state: state.clone(),
            modules,
            http,
        })
    }

    async fn state(&mut self) -> BotState {
        let mut module_states = HashMap::new();
        for module in &self.modules {
            module_states.insert(module.name().to_string(), module.state().await);
        }
        self.state.module_states.extend(module_states.into_iter());
        self.state.clone()
    }

    async fn shutdown(mut self) -> BotState {
        let mut module_states = HashMap::new();
        for module in self.modules {
            module_states.insert(module.name().to_string(), module.shutdown().await);
        }
        self.state.module_states.extend(module_states.into_iter());
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotState {
    enabled_modules: HashSet<String>,
    /// Serialized state of the modules
    module_states: HashMap<String, String>,
}
