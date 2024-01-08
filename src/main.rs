// Import bot commands
mod commands;

// Simple imports
use poise::serenity_prelude as serenity;
use tracing::{error, warn, info, debug};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::LevelFilter, prelude::*};
use std::{
    env,
    fs::create_dir,
};

struct Data {} // User data
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Create logs directory
    match create_dir("logs") {
        Ok(_) => (),
        Err(e) => error!("Failed to create logs directory: {e}"),
    }
    
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::HOURLY)
        .filename_prefix("protobot")
        .filename_suffix("log")
        .build("logs/")
        .expect("Failed to create logfile appender!");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let stdout_logger = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(LevelFilter::INFO);
    let file_logger = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .json()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(stdout_logger)
        .with(file_logger)
        .init();

    info!("Hello, ProtoBot here!"); 

    // Load variables from .env file
    dotenv::dotenv().ok();

    // Load bot token from environment variables
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    
    // Set bot intents
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
        commands: vec![commands::age(), commands::help(), commands::ping(), commands::about()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey protobot"),
                poise::Prefix::Literal("hey protobot,"),
            ],
            ..Default::default()
        },
        ..Default::default()
    })
    .token(token)
    .intents(intents)
    .setup(|ctx, _ready, framework| {
        Box::pin(async move {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data {})
        })
    });

    framework.run().await.unwrap();
}