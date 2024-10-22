use clap::Parser;
use cli::Args;
use color_eyre::Result;
use roobot::Bot;

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let bot = Bot::new(&args.data_dir, args.discord_token).await?;
    bot.run().await?;

    Ok(())
}
