use std::{collections::HashMap, str::FromStr};

use cron::Schedule;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId, MessageId, UserId};
use time::Date;

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
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
    pub(crate) queue: Vec<Update>,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Update {
    pub(crate) at: Date,
    pub(crate) message: MessageId,
    pub(crate) user: UserId,
}
