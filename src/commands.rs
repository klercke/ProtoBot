use crate::{Context, Error};

use poise::serenity_prelude as serenity;

/// Show the help menu
#[poise::command(prefix_command, invoke_on_edit, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom:
                "Additional documentation can be found at https://github.com/klercke/ProtoBot",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Pong!
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = "Pong!";
    ctx.say(response).await?;
    Ok(())
}

/// Sends a message containing the timestamp of when you or another user's account was created
#[poise::command(slash_command, prefix_command, invoke_on_edit)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.reply(response).await?;
    Ok(())
}

/// Prints some information about ProtoBot
#[poise::command(slash_command, prefix_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let mut response = String::from("ProtoBot v");
    response.push_str(env!("CARGO_PKG_VERSION"));
    response.push_str(". Source code and bug tracker: https://github.com/klercke/ProtoBot");
    ctx.say(response).await?;
    Ok(())
}

/// Spawns buttons to register and deregister commands
#[poise::command(slash_command, prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Deprecated
#[poise::command(prefix_command, hide_in_help)]
pub async fn correct(ctx: Context<'_>) -> Result<(), Error> {
    let response = "I'm sorry, but this command is not implemented in this version of Protobot.";
    ctx.reply(response).await?;
    Ok(())
}

/// Deprecated
#[poise::command(prefix_command, hide_in_help)]
pub async fn kick(ctx: Context<'_>) -> Result<(), Error> {
    let response = "I'm sorry, but this command is not implemented in this version of Protobot.";
    ctx.reply(response).await?;
    Ok(())
}

/// Deprecated
#[poise::command(prefix_command, hide_in_help)]
pub async fn nice(ctx: Context<'_>) -> Result<(), Error> {
    let response = "I'm sorry, but this command is not implemented in this version of Protobot.";
    ctx.reply(response).await?;
    Ok(())
}

/// Deprecated
#[poise::command(prefix_command, hide_in_help)]
pub async fn score(ctx: Context<'_>) -> Result<(), Error> {
    let response = "I'm sorry, but this command is not implemented in this version of Protobot.";
    ctx.reply(response).await?;
    Ok(())
}

/// Deprecated
#[poise::command(prefix_command, hide_in_help)]
pub async fn strange(ctx: Context<'_>) -> Result<(), Error> {
    let response = "I'm sorry, but this command is not implemented in this version of Protobot.";
    ctx.reply(response).await?;
    Ok(())
}

/// Deprecated
#[poise::command(prefix_command, hide_in_help)]
pub async fn tex(ctx: Context<'_>) -> Result<(), Error> {
    let response = "I'm sorry, but this command is not implemented in this version of Protobot.";
    ctx.reply(response).await?;
    Ok(())
}

/// Deprecated
#[poise::command(prefix_command, hide_in_help)]
pub async fn what(ctx: Context<'_>) -> Result<(), Error> {
    let response = "I'm sorry, but this command is not implemented in this version of Protobot.";
    ctx.reply(response).await?;
    Ok(())
}
