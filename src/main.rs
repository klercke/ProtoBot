mod commands;

use dotenv;
use poise::serenity_prelude as serenity;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Data {} // User data
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
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