use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;
use serenity::{
    all::{Context, EventHandler, GuildId, Http, Message, Ready},
    async_trait, Client,
};
use state::{ServerSettings, ServerState, State, Update};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{util::find_last_existing_msg, Load, Module};

mod state;

pub(crate) struct IconUpdaterLoader {
    client: Http,
}

impl Load for IconUpdaterLoader {
    type State = State;

    fn load(self, state: &Self::State) -> color_eyre::Result<impl Module> {
        Ok(IconUpdater::from_loder(self, state))
    }
}

pub(crate) struct IconUpdater {
    client: Http,
    settings: Arc<HashMap<GuildId, ServerSettings>>,
    state: Arc<Mutex<HashMap<GuildId, ServerState>>>,
    worker: JoinHandle<()>,
}

#[async_trait]
impl EventHandler for IconUpdater {
    async fn message(&self, ctx: Context, msg: Message) {
        let Some(guild_id) = msg.guild_id else {
            return;
        };
        let Some(settings) = self.settings.get(&guild_id) else {
            return;
        };
        if settings.channel != msg.channel_id {
            return;
        }

        let imgs = msg
            .attachments
            .iter()
            .filter(|att| att.width.is_some() && att.height.is_some())
            .collect_vec();
        if imgs.is_empty() {
            return;
        } else if imgs.len() > 1 {
            todo!("Sorry, only one image at a time please!")
        }

        let mut state = self.state.lock().await;
        let queue = &mut state.get_mut(&guild_id).expect("TODO").queue;

        // cleanup state and remove stale msgs
        //queue.iter().filter(|action| {
        //    self.client
        //        .get_message(settings.channel, action.message)
        //        .await
        //        .is_ok()
        //});

        if let Some(last) = queue.last() {
            if last.user == msg.author.id {
                todo!("No double-posting!");
            }
        }
        queue.push(Update {
            at: todo!(),
            user: msg.author.id,
            message: msg.id,
        });
        // TODO: post a nice confirmation message
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl IconUpdater {
    fn from_loder(loader: IconUpdaterLoader, state: &State) -> IconUpdater {
        let settings = Arc::from(state.settings.clone());
        let state = Arc::from(Mutex::from(state.state.clone()));
        let task = {
            let settings = Arc::clone(&settings);
            let state = Arc::clone(&state);
            tokio::spawn(async move {
                // go through servers and check if any are at a point where they need to update
                todo!()
            })
        };
        IconUpdater {
            client: loader.client,
            settings,
            state,
            worker: todo!(),
        }
    }
}

impl Module for IconUpdater {
    type State = State;

    async fn save(&self) -> Self::State {
        State {
            settings: (*self.settings).clone(),
            state: self.state.lock().await.clone(),
        }
    }

    async fn shutdown(self) -> Self::State {
        self.worker.abort();
        self.save().await
    }
}
