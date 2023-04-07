//! Define `Handler` struct and implement the `EventHandler` trait for it
//!
//! ## Interaction handling
//!
//! Implement the `interaction_create` method to handle incoming interactions
//! and delegate command handling to the appropriate functions from the `commands` module.
//!

use std::sync::{Arc, Mutex};
use rustc_hash::FxHashMap;
use serenity::{
    client::Context,
    model::{
        id::UserId,
				application::interaction::application_command::ApplicationCommandInteraction,

    },
};

use crate::{structures::{InteractionData, ApiResponse, Choice}, handlers::{Handler, HandlerStruct}};
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
	handler: &HandlerStruct,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
) {
  let prompt = command
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
    user_name, command.user.discriminator, prompt
  );
	// Generate the AI response and handle any errors
	let response = match generate_ai_response(handler, prompt, user_channel_key).await {
		Ok(response) => response,
		Err(e) => {
			error!("Error generating response: {:?}", e);
			return;
		}
	};
	let message = response.choices().first().unwrap().message().content.clone();

	if (edit_original_message_or_create_followup(
		ctx, 
		command, 
		interaction_data, 
		message.clone(), 
		handler.get_user_privacy(user_id)
	).await).is_err() {
		return;
	}
	handler.update_chat_history(user_channel_key, prompt, &message);
	
	
	
	// Update the usage statistics
	// todo: add a way to track usage for users 
	// handler.update_usage(user_id, prompt, &response, chat_history);
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
  handler: &HandlerStruct,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
) {
	let user_id = command.user.id;
	let channel_id = command.channel_id;

  // Remove the chat history for the user who called the command
  {
    let chat_histories = handler.chat_histories();
		chat_histories.lock().unwrap().remove(&(user_id, channel_id));
  }

	let chat_privacy = handler.get_user_privacy(user_id);

  let reset_message = "Chat history has been reset.".to_string();

  if (create_followup_message(ctx, command, reset_message, chat_privacy).await).is_err() {
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
  chat_privacy: &Arc<Mutex<FxHashMap<UserId, bool>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
) {
  set_chat_privacy(
    chat_privacy,
    true,
    ctx,
    command,
    interaction_data,
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
  chat_privacy: &Arc<Mutex<FxHashMap<UserId, bool>>>,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData
) {
  set_chat_privacy(
    chat_privacy,
    false,
    ctx,
    command,
    interaction_data,
  )
  .await;
}

// each personality has a different prompt
// we will use a match statement to handle the different personalities
// we will hold the personality and the prompt in a tuple (personality, prompt)
// we also need to handle the case where the user doesn't specify a personality
// we will use the default personality in that case
// we need this to change personalities through this command
// we will use a match statement to handle the different personalities
// we will hold the personality
// we also need to handle the case where the user doesn't specify a personality
// we will use the default personality in that case
// why would we need the privacy setting?
// we need to know if the chat is private or public
// but this only changes the prompt being used for the user
// we don't need to know if the chat is private or public
/// Handles the `/personality` command
/// 
/// Changes the personality of the AI
/// 
/// # Arguments
/// 
/// * `handler` - The Arc<Mutex<Handler>> containing the chat privacy settings
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// * `interaction_data` - The InteractionData containing interaction ID and token
/// 
/// # Example
// /// 
pub async fn personality_command(
	handler: &HandlerStruct,
	ctx: &Context,
	command: &ApplicationCommandInteraction,
) {
	let user = command.user.id;
	let channel = command.channel_id;
	let user_channel_key = (user, channel);
	let personality = command
		.data
		.options
		.get(0)
		.and_then(|opt| opt.value.as_ref())
		.and_then(|value| value.as_str())
		.unwrap_or("");

	// find the personality in the list of personas
	// if the personality is not found, use the default personality
	let personas = handler.personas();
	if let Some(persona) = personas.lock().unwrap().get(personality) {
		// set the personality for the user and channel
		handler.set_user_persona(user_channel_key, persona);
	} else {
		handler.set_user_persona(user_channel_key, "You are a helpful assistant.");
	}
	let personality_message = format!("Personality has been set to {}.", personality);

	if (create_followup_message(
		ctx, 
		command, 
		personality_message, 
		handler.get_user_privacy(command.user.id)
	).await).is_err() {
	}

	// we also want to give the admin the ability to add a custom prompt
	// we will use a match statement to handle the different cases
	// we will hold the prompt in a tuple (prompt, is_custom)
	// / Handles the `/prompt` command
}


// pub async fn personality_command(
// 	chat_privacy: &Arc<Mutex<HashMap<UserId, bool>>>,
// 	ctx: &Context,
// 	command: &ApplicationCommandInteraction,
// 	interaction_data: &InteractionData
// ) {
// 	let personality = command
// 		.data
// 		.options
// 		.get(0)
// 		.and_then(|opt| opt.value.as_ref())
// 		.and_then(|value| value.as_str())
// 		.unwrap_or("");


// 	.await;
// }
// 		// we also want to give the admin the ability to add a custom prompt
// 		// we will use a match statement to handle the different cases
// 		// we will hold the prompt in a tuple (prompt, is_custom)
// 		// / Handles the `/prompt` command
// 		// / 
// 		// / Sets the prompt of the AI to the specified prompt.
// 		// / 
// 		// / # Arguments
// 		// / 
// 		// / * `chat_privacy` - The Arc<Mutex<HashMap<UserId, bool>>> containing chat privacy settings
// 		// / * `ctx` - The Serenity Context for the command
// 		// / * `command` - The ApplicationCommandInteraction data
// 		// / * `interaction_data` - The InteractionData containing interaction ID and token
// 		// / 
// 		// / # Example
// 		// / 
// 		// / ```rust
// 		// / use serenity::model::interactions::InteractionData;
// 		// / use serenity::model::interactions::ApplicationCommandInteraction;
// 		// / use serenity::model::id::UserId;
// 		// / use serenity::client::Context;
// 		// / use std::sync::Arc;
// 		// / use std::sync::Mutex;
// 		// / use std::collections::HashMap;
// 		// / 
// 		// / 
// 		// / 