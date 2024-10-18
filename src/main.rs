use color_eyre::eyre::eyre;
use color_eyre::Result;
use roobot::{Bot, BotState};

#[tokio::main]
async fn main() -> Result<()> {
    let state: BotState = serde_json::from_str("").map_err(|e| eyre!("config blah"))?;

    let bot = Bot::new(&state, http);

    Ok(())
}
