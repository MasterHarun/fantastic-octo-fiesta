use serenity::{
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

use crate::{structures::{ApiResponse, Choice, Usage}, handlers::{ HandlerStruct}, users::{UserChatHistoryEntry, Personality, CommandState}};
use crate::utils::*;

/// Handles the `/chat` command
///
/// Generates an AI response based on the user's input and sends it as a follow-up message.
///
/// # Arguments
///
/// * `handler` - The Handler struct that contains the bot's state
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// 
pub async fn chat_command(
	handler: &HandlerStruct,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
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
	// if the user is not in the map, add them
	// log the user's prompt
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
	
	let chat_privacy = handler.with_user(user_id, |user| user.with_settings(|settings| settings.chat_privacy));

	if (edit_original_message_or_create_followup(
		ctx, 
		command, 
		message.clone(), 
		&chat_privacy.unwrap(),
	).await).is_err() {
		return;
	}

	let usage = response.usage();
	let total_tokens = usage.total_tokens();
	let prompt_tokens = usage.prompt_tokens();
	let completion_tokens = usage.completion_tokens();
	let combined_message = format!("user: {}\n ai: {}", prompt, message);

	let history_entry = UserChatHistoryEntry::new(
		combined_message, 
		prompt.to_owned(), 
		message, 
		total_tokens, 
		prompt_tokens, 
		completion_tokens
	);

	if !handler.user_exists(user_id) {
		handler.add_user(user_id);
	}
	
	handler.modify_user(user_id, |user| {
		let token_limit = user.with_settings(|settings| *settings.get_model().get_token_limit());
		user.modify_usage(|usage| {
			if !usage.contains_channel(channel_id) {
				usage.add_channel(channel_id);
			}
			usage.add_total_tokens(history_entry.get_total_tokens());
			usage.increase_chat_count();
			debug!("total user tokens: {:?}", usage.get_total_tokens());

			usage.modify_channel_data(channel_id, |channel_data| {
				channel_data.add_chat_history_entry(history_entry.clone());
				let user_tokens = channel_data.get_tokens_used();
				debug!("User usage: {:?}, token_limit: {:?}", user_tokens, token_limit);
				if user_tokens > &token_limit {
					channel_data.remove_oldest_entry();
				}
				
			});
		});
		
	}).unwrap_or_else(|e| {
		error!("Error modifying user: {:?}", e);
	});
	
}

/// Resets the chat history for the user and channel.
///
/// # Arguments
///
/// * `user` - The user to set the chat privacy for
/// * `ctx` - The `Context` for accessing the Discord API.
/// * `command` - The `ApplicationCommandInteraction` that triggered the reset command.
///
pub async fn reset_command(
  user: &HandlerStruct,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
) {
	let channel_id = command.channel_id;
	let user_id = command.user.id;

	user.modify_user(user_id, |user| {
		user.modify_usage(|usage| usage.reset_channel_usage(channel_id));
	}).unwrap_or_else(|e| {
		error!("Error modifying user: {:?}", e);
	});
	let chat_privacy = user.with_user(command.user.id, |user| user.with_settings(|settings| settings.chat_privacy));
	let chat_privacy = chat_privacy.unwrap();
  let reset_message = "Chat history has been reset.".to_string();
	
  if (create_followup_message(ctx, command, reset_message, &chat_privacy).await).is_err() {
	}
}

/// Handles the `/private` command
///
/// Sets the user's chat privacy to private, making the AI responses ephemeral.
///
/// # Arguments
///
/// * `user` - The user to set the chat privacy for
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// 
pub async fn private_command(
  user: &HandlerStruct,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
) {
  set_chat_privacy(
    user,
    true,
    ctx,
    command,
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
/// 
pub async fn public_command(
  user: &HandlerStruct,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
) {
  set_chat_privacy(
    user,
    false,
    ctx,
    command,
  )
  .await;
}


/// Handles the `/personality` command
/// 
/// Changes the personality of the AI
/// 
/// # Arguments
/// 
/// * `handler` - The Arc<Mutex<Handler>> containing the chat privacy settings
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// 
/// # Example
// /// 
pub async fn personality_command(
	// handler: &mut HandlerStruct,
	handler: &HandlerStruct,
	ctx: &Context,
	command: &ApplicationCommandInteraction,
) {
	let user_id = command.user.id;
	let personality_selected = command
			.data
			.options
			.get(0)
			.and_then(|opt| opt.value.as_ref())
			.and_then(|value| value.as_str())
			.unwrap_or("");


	let personas = handler.get_personas();
	if let Some(new_personality) = personas.iter().find(|p| p.name == personality_selected) {
		handler.modify_user(user_id, |user| {
			user.modify_settings(|settings| settings.set_personality(new_personality.clone()));
		}).unwrap_or_else(|e| {
			error!("Error modifying user: {:?}", e);
		});

		let message = format!(
			"You are now using the {} personality.",
			personality_selected
		);
		let chat_privacy = handler.with_user(user_id, |user| user.with_settings(|settings| settings.chat_privacy));
		let chat_privacy = chat_privacy.unwrap();
		if let Err(err) = create_followup_message(ctx, command, message, &chat_privacy).await {
			error!("Error sending follow-up message: {:?}", err);
		}
	}
}



/// Handles the `/add-personality` command
/// 
/// Adds a new personality to the bot
/// 
/// # Arguments
/// 
/// * `handler` - The Arc<Mutex<Handler>> containing the chat privacy settings
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// 
pub async fn add_personality_command(
	handler: &HandlerStruct,
	ctx: &Context,
	command: &ApplicationCommandInteraction,
) {
	let user_id = command.user.id;	

	let user_response = command
		.data
		.options
		.get(0)
		.and_then(|opt| opt.value.as_ref())
		.and_then(|value| value.as_str())
		.unwrap_or("");

	// let command_state = handler.with_user(user_id, |user| user.with_settings(|settings| settings.get_command_state()));
	let command_state = handler.with_user(user_id, |user| user.with_settings(|settings| settings.command_state.clone()));
	let command_state = command_state.unwrap();
	// todo: calculate the amount of tokens for the prompt
	let tokens = 0;
	match command_state {
		CommandState::None => {
			handler.modify_personas(|personas| {
				personas.push(Personality::new(user_response.to_string(), "".to_string(), tokens));
			}).unwrap_or_else(|err| error!("Error adding new personality: {:?}", err));
			handler.modify_user(user_id, |user| {
				user.modify_settings(|settings| settings.set_command_state(CommandState::PersonalityCommandState(user_response.to_string())));
			}).unwrap_or_else(|err| error!("Error setting command state: {:?}", err));

			let message = format!(
				"Personality {} has been created. What is the prompt for this personality?",
				user_response
			);
			let chat_privacy = handler.with_user(user_id, |user| user.with_settings(|settings| settings.chat_privacy));
			let chat_privacy = chat_privacy.unwrap();
			if let Err(err) = create_followup_message(ctx, command, message, &chat_privacy).await {
				error!("Error sending follow-up message: {:?}", err);
			}
		},
		CommandState::PersonalityCommandState(name) => {
			// create a new personality
			handler.modify_personas(|personas| {
				if let Some(personality) = personas.iter_mut().find(|p| p.name == *name) {
					personality.prompt = user_response.to_string();
					personality.tokens = tokens;
				}
			}).unwrap_or_else(|err| error!("Error modifying personality: {:?}", err));
			handler.modify_user(user_id, |user| {
				user.modify_settings(|settings| settings.set_command_state(CommandState::None));
			}).unwrap_or_else(|err| error!("Error setting command state: {:?}", err));

			let message = format!(
				"Personality has been created. The prompt {} has been set.",
				user_response
			);
			let chat_privacy = handler.with_user(user_id, |user| user.with_settings(|settings| settings.chat_privacy));
			let chat_privacy = chat_privacy.unwrap();
			if let Err(err) = create_followup_message(ctx, command, message, &chat_privacy).await {
				error!("Error sending follow-up message: {:?}", err);
			}
		},
	}
}