use std::{sync::{Arc, Mutex}, collections::HashMap};

use serenity::{
    client::Context,
    model::{
        id::UserId,
				application::interaction::application_command::ApplicationCommandInteraction, prelude::ChannelId,

    },
};

use crate::{structures::InteractionData};
use crate::utils::*;

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

  // Log the user's input with their name and ID
  info!(
    "User {}#{}: {}",
    user_name, command.user.discriminator, input
  );

  // Generate AI response here
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

  // Update the original message or send a follow-up message
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

  // Send a follow-up message to indicate the chat history has been reset
  let reset_message = "Chat history has been reset.".to_string();

  if let Err(_) = create_followup_message(&ctx, &command, reset_message, is_private).await {
    return;
  }
}

pub async fn private_command(
  chat_privacy: &Arc<Mutex<HashMap<UserId, bool>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
) {
	let user_id = command.user.id;
  set_chat_privacy(
    &chat_privacy,
    user_id,
    true,
    &ctx,
    &command,
    &interaction_data,
  )
  .await;
}

pub async fn public_command(
  chat_privacy: &Arc<Mutex<HashMap<UserId, bool>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData
) {
	let user_id = command.user.id;
  set_chat_privacy(
    chat_privacy,
    user_id,
    false,
    &ctx,
    &command,
    &interaction_data,
  )
  .await;
}
