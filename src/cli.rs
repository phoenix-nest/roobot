use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Roost-Bot Discord Bot
pub(crate) struct Args {
    #[arg(short, long, env = "ROOBOT_DATA_DIR")]
    /// Directory for roobot to store its state in. Must already exist
    pub(crate) data_dir: PathBuf,

    #[arg(long, env = "ROOBOT_DISCORD_TOKEN")]
    /// Discord token of the bot
    pub(crate) discord_token: String,
}
