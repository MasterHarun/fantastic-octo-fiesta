//! Define `Handler` struct and implement the `EventHandler` trait for it
//!
//! ## Interaction handling
//!
//! Implement the `interaction_create` method to handle incoming interactions
//! and delegate command handling to the appropriate functions from the `commands` module.
//!

use std::{sync::{Arc, Mutex}, collections::HashMap};

use serenity::{
    client::Context,
    model::{
        id::UserId,
				application::interaction::application_command::ApplicationCommandInteraction, prelude::ChannelId,

    },
};

use crate::structures::InteractionData;
use crate::utils::*;

/// Handles the `/chat` command
///
/// Generates an AI response based on the user's input and sends it as a follow-up message.
///
/// # Arguments
///
/// * `chat_histories` - The current chat history for the user and channel
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// * `is_private` - A boolean representing whether the chat is private or public
/// * `interaction_data` - The InteractionData containing interaction ID and token
/// 
pub async fn chat_command(
  chat_histories: &Arc<Mutex<HashMap<(UserId, ChannelId), String>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
  is_private: bool,
	interaction_data: &InteractionData,
) {
  let input = command
    .data
    .options
    .get(0)
    .and_then(|opt| opt.value.as_ref())
    .and_then(|value| value.as_str())
    .unwrap_or("");

	let user_id = command.user.id;
	let channel_id = command.channel_id;
	let user_channel_key = (user_id, channel_id);
	let user_name = command.user.name.clone();

  info!(
    "User {}#{}: {}",
    user_name, command.user.discriminator, input
  );

  let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found");
  let model = "gpt-3.5-turbo";

  let chat_history = {
    let mut chat_histories = chat_histories.lock().unwrap();
    chat_histories
      .entry(user_channel_key)
      .or_insert_with(String::new)
      .clone()
  };

  let ai_response =
    generate_ai_response(input, model, &api_key, user_channel_key, chat_history).await;

  if let Some(response) = ai_response.clone() {
    if let Err(_) = edit_original_message_or_create_followup(
      ctx,
      command,
      interaction_data,
      response,
      is_private,
    )
    .await
    {
      return;
    }
  } else {
    error!("Error: AI response could not be generated.");
    return;
  }

  // Update the chat history with the user's input and the AI's response
  {
    let mut chat_histories = chat_histories.lock().unwrap();
    let history = chat_histories
      .entry(user_channel_key)
      .or_insert_with(String::new);
    history.push_str(&format!("User: {}\n", input));

    if let Some(ref response) = ai_response {
      history.push_str(&format!("AI: {}\n", response));
    }

    // Truncate the chat history to a certain number of tokens
    let TOKEN_LIMIT = 4096;
    // debug!("Chat history before truncation:\n{}", history);
    truncate_chat_history(history, TOKEN_LIMIT);
    // debug!("Chat history after truncation:\n{}", history);
  }
}

/// Resets the chat history for the user and channel.
///
/// # Arguments
///
/// * `chat_histories` - A mutable reference to the chat history to be reset.
/// * `ctx` - The `Context` for accessing the Discord API.
/// * `command` - The `ApplicationCommandInteraction` that triggered the reset command.
/// * `is_private` - A boolean representing whether the chat is private or public
///
pub async fn reset_command(
  chat_histories: &Arc<Mutex<HashMap<(UserId, ChannelId), String>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
  is_private: bool,
) {
	let user_id = command.user.id;
	let channel_id = command.channel_id;

  // Remove the chat history for the user who called the command
  {
    let mut chat_histories = chat_histories.lock().unwrap();
    chat_histories.remove(&(user_id, channel_id));
  }

  let reset_message = "Chat history has been reset.".to_string();

  if let Err(_) = create_followup_message(&ctx, &command, reset_message, is_private).await {
    return;
  }
}

/// Handles the `/private` command
///
/// Sets the user's chat privacy to private, making the AI responses ephemeral.
///
/// # Arguments
///
/// * `chat_privacy` - The Arc<Mutex<HashMap<UserId, bool>>> containing chat privacy settings
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// * `interaction_data` - The InteractionData containing interaction ID and token
/// 
pub async fn private_command(
  chat_privacy: &Arc<Mutex<HashMap<UserId, bool>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
) {
  set_chat_privacy(
    &chat_privacy,
    true,
    &ctx,
    &command,
    &interaction_data,
  )
  .await;
}

/// Handles the `/public` command
///
/// Sets the user's chat privacy to public, making the AI responses visible to everyone.
///
/// # Arguments
///
/// * `chat_privacy` - The Arc<Mutex<HashMap<UserId, bool>>> containing chat privacy settings
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// * `interaction_data` - The InteractionData containing interaction ID and token
/// 
pub async fn public_command(
  chat_privacy: &Arc<Mutex<HashMap<UserId, bool>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData
) {
  set_chat_privacy(
    chat_privacy,
    false,
    &ctx,
    &command,
    &interaction_data,
  )
  .await;
}
