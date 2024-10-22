use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use color_eyre::eyre::{eyre, Context, Result};
use data::Datastore;
use icon_updater::IconUpdater;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{EventHandler, GatewayIntents, Http},
    Client,
};
use tokio::time::sleep;
use tracing::{error, instrument};

mod data;
mod icon_updater;
mod util;

type State = String;

#[async_trait]
pub(crate) trait Module: EventHandler + std::fmt::Debug {
    /// the name of the module, must be unique
    fn name(&self) -> &'static str;

    /// Setup the module, registering handlers and commands
    fn load(state: Option<&str>) -> Result<Self>
    where
        Self: Sized;

    /// Perform additional tasks once the bot has started up.
    /// This is for tasks that require a bot http client.
    #[allow(unused_variables)]
    async fn post_init(&mut self, http: Arc<Http>) -> Result<()> {
        Ok(())
    }
    /// Save the state of the module and return a serialized form
    fn save(&self) -> State;
    /// Shutdown the module, returning the final state and stopping all tasks
    fn shutdown(self: Box<Self>) -> State;
}

pub struct Bot {
    datastore: Datastore,
    state: BotState,
    modules: Vec<Box<dyn Module>>,
    client: Client,
}
impl Bot {
    pub async fn new(data_dir: &Path, token: String) -> Result<Bot> {
        let datastore = Datastore::new(data_dir);
        let state: BotState =
            serde_json::from_str(&datastore.load_state_file()?).wrap_err("Parsing bot state")?;

        let modules: Vec<Box<dyn Module>> = state
            .enabled_modules
            .iter()
            .map(|module_name| match module_name.as_str() {
                icon_updater::NAME => IconUpdater::load(
                    state
                        .module_states
                        .get(module_name.as_str())
                        .map(String::as_str),
                )
                .map(|module| Box::new(module) as Box<dyn Module>)
                .map_err(|e| eyre!("Error loading module {module_name}: {e}")),
                unknown => Err(eyre!("Unknown module name {unknown}")),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;
        let client = Client::builder(token, intents)
            .await
            .wrap_err("Creating discord client")?;

        Ok(Bot {
            datastore,
            state: state.clone(),
            modules,
            client,
        })
    }

    #[instrument(skip(self))]
    pub async fn run(mut self) -> Result<()> {
        for module in self.modules.iter_mut() {
            module.post_init(Arc::clone(&self.client.http)).await?;
        }

        loop {
            sleep(Duration::from_secs(30)).await;
            self.save().await;
            if let Err(e) = self
                .datastore
                .save_state_file(&serde_json::to_string(&self.state)?)
            {
                error!(error = "Unable to save state", err = ?e, state = ?self.state);
            }
        }
    }

    async fn save(&mut self) {
        let mut module_states = HashMap::new();
        for module in &self.modules {
            module_states.insert(module.name().to_string(), module.save());
        }
        self.state.module_states.extend(module_states.into_iter());
    }

    async fn shutdown(mut self) -> BotState {
        let mut module_states = HashMap::new();
        for module in self.modules {
            module_states.insert(module.name().to_string(), module.shutdown());
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
