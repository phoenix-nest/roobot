use clap::Parser;
use cli::Args;
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use data::Datastore;
use roobot::{Bot, BotState};
use serenity::all::GatewayIntents;
use serenity::Client;

mod cli;
mod data;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let datastore = Datastore::new(&args.data_dir);

    let state: BotState =
        serde_json::from_str(&datastore.load_state_file()?).wrap_err("Parsing bot state")?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(args.discord_token, intents)
        .await
        .wrap_err("Creating discord client")?;

    let bot = Bot::new(&state, http);

    Ok(())
}
