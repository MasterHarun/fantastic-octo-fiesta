use serenity::{
  client::Context,
  model::application::interaction::application_command::ApplicationCommandInteraction,
};

use crate::utils::*;
use crate::{
  handlers::HandlerStruct,
  structures::{ApiResponse, Choice, Usage},
  users::{Personality, UserChatHistoryEntry},
};

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
  let message = response
    .choices()
    .first()
    .unwrap()
    .message()
    .content
    .clone();

  let chat_privacy = handler.with_user(user_id, |user| {
    user.with_settings(|settings| settings.chat_privacy)
  });

  if (edit_original_message_or_create_followup(
    ctx,
    command,
    message.clone(),
    &chat_privacy.unwrap(),
  )
  .await)
    .is_err()
  {
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
    completion_tokens,
  );

  if !handler.user_exists(user_id) {
    handler.add_user(user_id);
  }

  handler
    .modify_user(user_id, |user| {
      let token_limit = user.with_settings(|settings| *settings.get_model().get_token_limit());
      user.modify_usage(|usage| {
        if !usage.contains_channel(channel_id) {
          usage.add_channel(channel_id);
        }
        // ?? why is this here?
        // !? The only time the amount of tokens a user has used is at chat time when they are sent
        // !? Even if the system message is changed by the personality command, it will still be the same amount of tokens
        usage.add_total_tokens(history_entry.get_total_tokens());
        usage.increase_chat_count();
        debug!("total user tokens: {:?}", usage.get_total_tokens());

        usage.modify_channel_data(channel_id, |channel_data| {
          channel_data.add_chat_history_entry(history_entry.clone());
          let user_tokens = channel_data.get_tokens_used();
          debug!(
            "User usage: {:?}, token_limit: {:?}",
            user_tokens, token_limit
          );
          if user_tokens > &token_limit {
            channel_data.remove_oldest_entry();
          }
        });
      });
    })
    .unwrap_or_else(|e| {
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

  user
    .modify_user(user_id, |user| {
      user.modify_usage(|usage| usage.reset_channel_usage(channel_id));
    })
    .unwrap_or_else(|e| {
      error!("Error modifying user: {:?}", e);
    });
  let chat_privacy = user.with_user(command.user.id, |user| {
    user.with_settings(|settings| settings.chat_privacy)
  });
  let chat_privacy = chat_privacy.unwrap();
  let reset_message = "Chat history has been reset.".to_string();

  if (create_followup_message(ctx, command, reset_message, &chat_privacy).await).is_err() {}
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
  set_chat_privacy(user, true, ctx, command).await;
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
  set_chat_privacy(user, false, ctx, command).await;
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
  handler: &HandlerStruct,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
) {
  // debug!("Personality command: {:?}", command);
  // fixme: The first message after changing the personality isnt set to the new personality
  let user_id = command.user.id;
  let personas = handler.get_personas();

  debug!("Personality command: {:#?}", command);
  let new_personality = command
    .data
    .options
    .get(0)
    .and_then(|option| option.value.as_ref())
		.and_then(|value| value.as_str())
		.unwrap_or("default");
  let new_personality = new_personality;
  for persona in personas {
    if persona.name == new_personality {
      handler
        .modify_user(user_id, |user| {
          user.modify_settings(|settings| settings.set_personality(persona.clone()));
          // info!("Personality command selected: {:?}", persona.name)
        })
        .unwrap_or_else(|e| {
          error!("Error modifying user: {:?}", e);
        });
    }
  }

  let message = format!("You are now using the {:?} personality.", new_personality);
  let chat_privacy = handler.with_user(user_id, |user| {
    user.with_settings(|settings| settings.chat_privacy)
  });
  let chat_privacy = chat_privacy.unwrap();
  if let Err(err) = create_followup_message(ctx, command, message, &chat_privacy).await {
    error!("Error sending follow-up message: {:?}", err);
  }
}

pub async fn persona_control_command(
	handler: &HandlerStruct,
	ctx: &Context,
	command: &ApplicationCommandInteraction,
) {
	let user_id = command.user.id;
	debug!("Persona control command: {:#?}", command);
	let name = command.data.options.get(0).unwrap().name.as_str();
	let mut message = Default::default();
	match name {
		"add" => {
			let command_data = command.data.options.get(0).unwrap();
			debug!("Name: {:#?}", name);
			let name = command_data
				.options
				.get(0)
				.and_then(|opt| opt.value.as_ref())
				.and_then(|value| value.as_str())
				.unwrap_or("");
			debug!("Name: {:#?}", name);
			let description = command_data
				.options
				.get(1)
				.and_then(|opt| opt.value.as_ref())
				.and_then(|value| value.as_str())
				.unwrap_or("");
			debug!("Description: {:#?}", description);
			let prompt = command_data
				.options
				.get(2)
				.and_then(|opt| opt.value.as_ref())
				.and_then(|value| value.as_str())
				.unwrap_or("");
			debug!("Prompt: {:#?}", prompt);

			handler
				.modify_personas(|personas| {
					if let Some(personality) = personas.iter_mut().find(|p| p.name == *name) {
						personality.prompt = prompt.to_string();
						personality.description = description.to_string();
						// personality.tokens = tokens;
					} else {
						personas.push(Personality {
							name: name.to_string(),
							description: description.to_string(),
							prompt: prompt.to_string(),
							tokens: 0,
						});
					}
				})
				.unwrap_or_else(|err| error!("Error modifying personality: {:?}", err));

			message = format!(
				"Personality {} has been created.",
				name
			);
			
		}
		"remove" => {
			let name = command.data.options.get(0).unwrap().options.get(0).unwrap();
			debug!("Name: {:#?}", name);
			let name = name
				.options
				.get(0)
				.and_then(|opt| opt.value.as_ref())
				.and_then(|value| value.as_str())
				.unwrap_or("");
			debug!("Name: {:#?}", name);

			handler
				.modify_personas(|personas| {
					personas.retain(|p| p.name != *name);
				})
				.unwrap_or_else(|err| error!("Error modifying personality: {:?}", err));

			message = format!(
				"Personality {} has been deleted.",
				name
			);
			let command_id = handler.get_command_id("persona-control").await.unwrap();
			// ?? remove the old command
			let _ = ctx.http.delete_global_application_command(command_id).await;
			// ?? create the new command
			// let _ = register_application_commands(handler, &ctx.http).await;
		}
		_ => {},
	}
	let command_id = handler.get_command_id("personality").await.unwrap();
	// ?? remove the old command
	let _ = ctx.http.delete_global_application_command(command_id).await;
	// ?? create the new command
	let _ = register_application_commands(handler, &ctx.http).await;

	let chat_privacy = handler.with_user(user_id, |user| {
		user.with_settings(|settings| settings.chat_privacy)
	});
	let chat_privacy = chat_privacy.unwrap();

	if let Err(err) = create_followup_message(ctx, command, message, &chat_privacy).await {
		error!("Error sending follow-up message: {:?}", err);
	}
}
