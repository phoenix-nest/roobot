use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId, MessageId, UserId};

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize, Default)]
pub(crate) struct State {
    pub(crate) settings: HashMap<GuildId, ServerSettings>,
    pub(crate) state: HashMap<GuildId, ServerState>,
}

// cron does not have a serde implementation ...
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ScheduleSerde(String);
impl From<Schedule> for ScheduleSerde {
    fn from(value: Schedule) -> Self {
        ScheduleSerde(value.to_string())
    }
}
impl FromStr for ScheduleSerde {
    type Err = cron::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Schedule::from_str(s).map(|s| ScheduleSerde(s.to_string()))
    }
}
impl ScheduleSerde {
    pub(crate) fn schedule(&self) -> Schedule {
        Schedule::from_str(&self.0).unwrap()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ServerSettings {
    pub(crate) channel: ChannelId,
    pub(crate) schedule: ScheduleSerde,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize, Default)]
pub(crate) struct ServerState {
    pub(crate) next_update: DateTime<Utc>,
    pub(crate) queue: VecDeque<Update>,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Update {
    pub(crate) message: MessageId,
    pub(crate) user: UserId,
}
