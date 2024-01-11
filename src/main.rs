// Import bot commands
mod commands;

// Imports
use poise::serenity_prelude::{self as serenity, ActivityData};
use tracing::{error, info, debug};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{filter::LevelFilter, prelude::*};
use regex::Regex;
use std::{
    env,
    fs::create_dir,
};
use rand::{
    Rng,
    SeedableRng,
    rngs::SmallRng,
};

// Types used by command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data to pass to command functions
struct Data {
} 


#[tokio::main]
async fn main() {
    // Create logs directory
    match create_dir("logs") {
        Ok(_) => (),
        Err(e) => error!("Failed to create logs directory: {e}"),
    }
    
    // Set up logging
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

    // Say hi :)
    info!("Hello, ProtoBot here!"); 

    // Load variables from .env file
    dotenv::dotenv().ok();

    // Load bot token from environment variables
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    
    // Set bot intents
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    
    // Build the framework for the bot
    let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
        commands: vec![
            commands::age(),
            commands::help(),
            commands::ping(),
            commands::about(),
            commands::register(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey protobot,"),
                poise::Prefix::Literal("hey protobot"),
            ],
            ..Default::default()
        },
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
        Box::pin(async move {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data {
            })
        })
    })
    .build();
    debug!("Framework building done");
    
    // Build client from framework
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    debug!("Client creation done!");

    // Build and run the framework
    client.unwrap().start().await.unwrap();
    debug!("Client started!");

}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("Successfully authenticated to Discord as {}", data_about_bot.user.name);
            
            // Set Discord status for the bot
            let bot_status_message = format!("ProtoBot v{}: Rewritten in Rust!", env!("CARGO_PKG_VERSION"));
            ctx.set_activity(Some(ActivityData::custom(&bot_status_message)));
            info!("Set bot status to \"{}\"", bot_status_message);
        }
        serenity::FullEvent::Message { new_message } => {
            // Tell the bot to ignore its own messages (to prevent loops)
            if new_message.is_own(ctx) {
                debug!("Ignoring message sent by self");
                return Ok(());
            }

            // Hi x, I'm dad!
            // This will capture any text after "i am", "i'm", or "im", stopping the capture on punctuation or a newline 
            let im_dad_regex = Regex::new(r#"(?i)(?:\b|^)(?:i['´`‘’]?m|i am)\b(.+?)(?:[\n.,;!?]|$)"#).unwrap();
            // Dad jokes have a 1 in dad_joke_chance chance of ocurring
            let dad_joke_chance = 10;
            let mut dad_joke_rng = SmallRng::from_entropy();
            if dad_joke_rng.gen_range(1 ..= dad_joke_chance) == 1 {
                if let Some(caps) = im_dad_regex.captures(&new_message.content) {
                    let captured_text = caps.get(1).map_or("", |m| m.as_str().trim());
                    info!("Found dad joke: I'm {}", captured_text);
                    new_message.reply(ctx, format!("Hi {}, I'm dad!", captured_text)).await?;
                }
            }
            
            // Happy birthday messages
            if new_message.content.to_lowercase().contains("happy birthday") {
                for user in &new_message.mentions {
                    info!("Wishing happy birthday to {} ({})", user.name, user.id);
                    new_message.channel_id.say(&ctx.http, format!("Happy birthday <@{}>! 🎈🎉🎂", user.id)).await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
