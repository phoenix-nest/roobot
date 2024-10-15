use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io::Cursor;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use chrono::{Duration, Utc};
use color_eyre::eyre::eyre;
use color_eyre::eyre::{self, Result};
use img::process_icon;
use itertools::Itertools;
use message::MessageExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateMessage, MessageReference};
use serenity::{
    all::{Context, EventHandler, GuildId, Http, HttpError, Message, Ready, StatusCode},
    async_trait,
};
use state::{ServerSettings, ServerState, State, Update};
use tokio::time::sleep;
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{error, info, instrument, warn, Span};

use crate::{util::send_or_log, Load, Module};

mod img;
mod message;
mod state;

pub(crate) struct IconUpdaterLoader {
    client: Http,
}

impl Load for IconUpdaterLoader {
    type State = State;

    fn load(self, state: &Self::State) -> color_eyre::Result<impl Module> {
        Ok(IconUpdater::from_loader(self, state))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct UpdateIconPayload {
    icon: String,
}
#[derive(Debug)]
struct IconUpdateTask {
    client: Arc<Http>,
    guild_id: GuildId,
    settings: ServerSettings,
    // global state, todo: separate out into per-guild mutex
    state: Arc<Mutex<HashMap<GuildId, ServerState>>>,
}
impl IconUpdateTask {
    #[instrument(skip(self), fields(self.client_id, self.settings, state))]
    async fn update_icon(&self) -> Result<()> {
        let now = Utc::now();
        let mut state_guard = self.state.lock().await;
        let state = &mut match state_guard.entry(self.guild_id) {
            Vacant(entry) => entry.insert(ServerState::default()),
            Occupied(entry) => entry.into_mut(),
        };
        Span::current().record("state", format!("{:?}", state));
        if now <= state.next_update {
            // now is not the time
            return Ok(());
        }

        let update = {
            loop {
                let Some(update) = state.queue.front() else {
                    break None;
                };
                match self
                    .client
                    .get_message(self.settings.channel, update.message)
                    .await
                {
                    Ok(msg) => {
                        let img = msg.images().into_iter().collect_vec().first().cloned();
                        if let Some(img) = img {
                            break Some((update, img.to_owned()));
                        };
                        // Message does not have image anymore, skipping
                        continue;
                    }
                    Err(serenity::Error::Http(HttpError::UnsuccessfulRequest(e)))
                        if e.status_code == StatusCode::NOT_FOUND =>
                    {
                        // Message was deleted
                        state.queue.pop_front();
                    }
                    Err(err) => {
                        warn!(
                            error = "Could not validate message in server queue, skipping",
                            msg = update.message.to_string(),
                            err = err.to_string()
                        );
                        continue;
                    }
                }
            }
        };
        let Some((update, image)) = update else {
            // No images available
            return Ok(());
        };
        let data = match image.download().await {
            Ok(data) => data,
            Err(err) => {
                error!(
                    error = "Could not download icon image",
                    err = err.to_string()
                );
                return Err(eyre!("Could not download icon image"));
            }
        };
        let icon = match process_icon(data) {
            Ok(i) => i,
            Err(err) => {
                error!(
                    error = "Could not process icon image, skipping",
                    err = err.to_string()
                );
                let errmsg = CreateMessage::new().content("Sorry, I was unable to use this icon. Please resubmit it or contact an admin").reference_message(MessageReference::from((self.settings.channel, update.message)));
                send_or_log(self.settings.channel.send_message(&self.client, errmsg)).await;
                state.queue.pop_front();
                return Err(eyre!("Unprocessable image"));
            }
        };
        if let Err(err) = self
            .client
            .edit_guild(
                self.guild_id,
                &UpdateIconPayload { icon },
                Some("roobot automatic server icon update"),
            )
            .await
        {
            error!(error = "Unable to update icon", ?err)
        }
        info!("Updated icon");
        // maybe post a nice message here
        return Ok(());
    }
}

#[derive(Debug)]
pub(crate) struct IconUpdater {
    client: Arc<Http>,
    settings: HashMap<GuildId, ServerSettings>,
    state: Arc<Mutex<HashMap<GuildId, ServerState>>>,
    tasks: HashMap<GuildId, JoinHandle<()>>,
}

impl IconUpdater {
    fn from_loader(loader: IconUpdaterLoader, state: &State) -> IconUpdater {
        let settings = state.settings.clone();
        let http = Arc::new(loader.client);
        let state = Arc::from(Mutex::from(state.state.clone()));
        let tasks = {
            let settings = settings.clone();
            settings
                .into_iter()
                .map(|(id, settings)| {
                    let state = Arc::clone(&state);
                    let http = Arc::clone(&http);
                    (
                        id,
                        tokio::spawn(async move {
                            let mut rng = rand::rngs::OsRng;
                            let task = IconUpdateTask {
                                client: http,
                                guild_id: id,
                                settings: settings.clone(),
                                state,
                            };
                            loop {
                                sleep(std::time::Duration::from_secs(
                                    // distribute requests over half an hour
                                    1800 + rng.gen_range(0..1800),
                                ))
                                .await;
                                if let Err(err) = task.update_icon().await {
                                    error!(error = "Icon update failed", guild = ?id, ?err);
                                };
                            }
                        }),
                    )
                })
                .collect::<HashMap<_, _>>()
        };
        IconUpdater {
            client: http,
            settings: settings.clone(),
            state,
            tasks,
        }
    }
}

#[async_trait]
impl EventHandler for IconUpdater {
    #[instrument(skip(ctx), fields(guild_id, settings))]
    async fn message(&self, ctx: Context, msg: Message) {
        let Some(guild_id) = msg.guild_id else {
            return;
        };
        Span::current().record("guild_id", guild_id.to_string());
        let Some(settings) = self.settings.get(&guild_id) else {
            return;
        };
        Span::current().record("settings", format!("{:?}", &settings));
        if settings.channel != msg.channel_id {
            return;
        }

        let imgs = msg.images().into_iter().collect_vec();
        let img = match imgs.len() {
            0 => return,
            1 => imgs.first().unwrap(),
            _ => {
                send_or_log(msg.reply_ping(
                    ctx,
                    "Sorry, I cannot process messages with multiple images 😣.
                        Please submit only a single icon at a time.",
                ))
                .await;
                return;
            }
        };

        let mut state_lock = self.state.lock().await;
        let state = &mut match state_lock.entry(guild_id) {
            Vacant(entry) => entry.insert(ServerState::default()),
            Occupied(entry) => entry.into_mut(),
        };

        // ensure the last update in our queue is still valid so we can validate it
        let last_valid_message = {
            loop {
                let Some(update) = state.queue.back() else {
                    break None;
                };
                match self
                    .client
                    .get_message(settings.channel, update.message)
                    .await
                {
                    Ok(msg) => break Some(msg), // found valid message
                    Err(serenity::Error::Http(HttpError::UnsuccessfulRequest(e)))
                        if e.status_code == StatusCode::NOT_FOUND =>
                    {
                        // Message was deleted
                        state.queue.pop_back();
                    }
                    Err(e) => {
                        warn!(
                            error = "Could not validate message in server queue",
                            msg = update.message.to_string(),
                            err = e.to_string()
                        );
                        break None;
                    }
                }
            }
        };

        if let Some(last) = last_valid_message {
            if last.author.id == msg.author.id
                && (*msg.timestamp).signed_duration_since(*last.timestamp) < Duration::minutes(60)
            {
                send_or_log(msg.reply_ping(
                    ctx,
                    "Sorry, you have already submitted an image within the last hour.\n
                              If you would like to replace your previous submission,
                              just delete both messages and try again",
                ))
                .await;
                return;
            }
        }
        let Some(schedule_at) = settings
            .schedule
            .schedule()
            .upcoming(Utc)
            .take(state.queue.len())
            .next()
        else {
            error!("Cron expression did not yield valid time");
            send_or_log(msg.reply_ping(ctx, "Sorry, something is wrong with this servers update schedule. Please contact an admin.")).await;
            return;
        };

        let data = match img.download().await {
            Ok(data) => data,
            Err(err) => {
                error!(
                    error = "Could not download icon image",
                    err = err.to_string()
                );
                send_or_log(msg.reply_ping(&ctx, "Sorry, something went wrong when trying to check the image. Please try again or contact an admin.")).await;
                return;
            }
        };
        if let Err(err) = process_icon(data) {
            error!(
                error = "Could not process icon image",
                err = err.to_string()
            );
            send_or_log(msg.reply_ping(&ctx, "Sorry, something went wrong when trying to check the image. Please try again or contact an admin.")).await;
        }
        state.queue.push_back(Update {
            user: msg.author.id,
            message: msg.id,
        });
        send_or_log(msg.react(&ctx, '👍')).await;
        send_or_log(msg.reply_ping(
            &ctx,
            format!("Awesome! Your icon will be set on {}", schedule_at),
        ))
        .await;
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

impl Module for IconUpdater {
    type State = State;

    async fn save(&self) -> Self::State {
        State {
            settings: self.settings.clone(),
            state: self.state.lock().await.clone(),
        }
    }

    async fn shutdown(self) -> Self::State {
        for task in self.tasks.values() {
            task.abort();
        }
        self.save().await
    }
}
