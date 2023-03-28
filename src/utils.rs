//! Contains utility functions to support the main functionality of the bot
//!
//! ## Utility functions
//!
//! - `register_application_commands`: Registers application commands with Discord
//! - `generate_ai_response`: Generates an AI response using the OpenAI API
//! - `truncate_chat_history`: Truncates chat history to a specified number of tokens
//! - `acknowledge_interaction`: Acknowledges an interaction with Discord
//! - `create_followup_message`: Sends a follow-up message for an interaction
//! - `edit_original_message_or_create_followup`: Edits the original interaction message or creates a follow-up message
//! - `set_chat_privacy`: Sets chat privacy for a user
//!

use serde_json::json;
use serenity::{model::prelude::{interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}, UserId, ChannelId, command::{CommandOptionType, Command}}, prelude::Context, http::Http};
use tokio::time::{timeout, Duration};
use unicode_segmentation::UnicodeSegmentation;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap};

use crate::structures::*;

/// Creates a follow-up message in response to an application command (slash command).
///
/// # Arguments
///
/// * `ctx` - The `Context` for accessing the Discord API.
/// * `command` - The `ApplicationCommandInteraction` that triggered the follow-up message.
/// * `content` - The content of the follow-up message.
/// * `chat_privacy` - A boolean representing the privacy setting of the message.
///
pub async fn create_followup_message(
  ctx: &Context,
  command: &ApplicationCommandInteraction,
  content: String,
  chat_privacy: bool,
) -> Result<(), ()> {
  match command
    .create_followup_message(&ctx.http, |message| {
      if chat_privacy {
        debug!("Chat privacy passed: {}", chat_privacy);
        message.ephemeral(true).content(content)
      } else {
        message.content(content)
      }
    })
    .await
  {
    Ok(_) => {
      debug!("Sent the follow-up message");
      Ok(())
    }
    Err(why) => {
      error!("Error sending follow-up message: {:?}", why);
      Err(())
    }
  }
}

/// Edits the original message or creates a follow-up message
///
/// Edits the original interaction response message or creates a new follow-up message with the specified content.
///
/// # Arguments
///
/// * `ctx` - The Serenity Context
/// * `command` - The ApplicationCommandInteraction data
/// * `interaction_data` - The InteractionData containing interaction ID and token
/// * `content` - The content of the message
/// * `is_private` - A boolean representing whether the chat is private or public
/// 
pub async fn edit_original_message_or_create_followup(
	ctx: &Context,
	command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
	content: String,
	is_private: bool,
) -> Result<(), ()> {
	let _interaction_id = &interaction_data.interaction_id;
	let response_token = &interaction_data.response_token;

	let message = if is_private {
			serde_json::json!({
					"content": content,
					"flags": 64
			})
	} else {
			serde_json::json!({
					"content": content
			})
	};

	if let Ok(_) = ctx
			.http
			.edit_original_interaction_response(response_token, &message)
			.await
	{
			debug!("Edited the original message");
			return Ok(());
	} else {
			if let Err(why) = create_followup_message(ctx, command, content, is_private).await {
					error!("Error sending follow-up message: {:?}", why);
					return Err(());
			}
			debug!("Sent a follow-up message");
			return Ok(());
	}
}

/// Acknowledges an interaction
///
/// Sends an acknowledgement response to the interaction and returns the interaction data.
///
/// # Arguments
///
/// * `command` - The ApplicationCommandInteraction data
/// * `ctx` - The Serenity Context for the command
/// * `ephemeral` - A boolean indicating whether the acknowledgement message should be ephemeral
/// 
pub async fn acknowledge_interaction(
  command: &ApplicationCommandInteraction,
  ctx: &Context,
  ephemeral: bool,
) -> Result<InteractionData, ()> {
  match timeout(
    Duration::from_millis(2500),
    command.create_interaction_response(&ctx.http, |response| {
      if ephemeral {
        response
          .kind(InteractionResponseType::ChannelMessageWithSource)
          .interaction_response_data(|message| message.ephemeral(true).content("Processing..."))
      } else {
        response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
      }
    }),
  )
  .await
  {
    Ok(Ok(_)) => {
      debug!("Acknowledged the interaction");
      Ok(InteractionData {
        interaction_id: command.id.to_string(),
        response_token: command.token.clone(),
      })
    }
    Ok(Err(why)) => {
      error!("Error acknowledging interaction: {:?}", why);
      Err(())
    }
    Err(_) => {
      error!("Timed out acknowledging interaction");
      Err(())
    }
  }
}

/// Truncates the chat history to fit within the character limit imposed by the OpenAI API.
///
/// # Arguments
///
/// * `chat_history` - The current chat history as a String.
/// * `input` - The user's input that will be added to the chat history.
/// * `max_length` - The maximum allowed length of the chat history, including the user's input.
///
pub fn truncate_chat_history(chat_history: &mut String, max_tokens: usize) {
  let tokens: Vec<&str> = chat_history.unicode_words().collect();
  let token_count = tokens.len();

  debug!("Current chat history token count: {}", token_count);

  if token_count > max_tokens {
    let tokens_to_remove = token_count - max_tokens;
    let new_history: String = tokens[tokens_to_remove..].join(" ");
    *chat_history = new_history;
    debug!(
      "Chat history has been truncated by {} tokens",
      tokens_to_remove
    );
  } else {
    debug!("Chat history is within the token limit");
  }
}

/// Sets chat privacy for a user
///
/// Updates the chat privacy settings for a user and sends a follow-up message to indicate the change.
///
/// # Arguments
///
/// * `chat_privacy_map` - The Arc<Mutex<HashMap<UserId, bool>>> containing chat privacy settings
/// * `chat_privacy` - A boolean representing the new chat privacy setting
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
/// * `interaction_data` - The InteractionData containing interaction ID and token
/// 
pub async fn set_chat_privacy(
	chat_privacy_map: &Arc<Mutex<HashMap<UserId, bool>>>,
	chat_privacy: bool,
	ctx: &Context,
	command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
) {
	let user_id = command.user.id;
	chat_privacy_map
			.lock()
			.unwrap()
			.insert(user_id, chat_privacy);
	let response = if chat_privacy {
			"Chat privacy set to private.".to_string()
	} else {
			"Chat privacy set to public.".to_string()
	};

	if let Err(_) = edit_original_message_or_create_followup(
			&ctx,
			&command,
			interaction_data,
			response,
			chat_privacy,
	)
	.await
	{
			return;
	}
}


/// Generates an AI response using the OpenAI API based on the user input and chat history.
///
/// # Arguments
///
/// * `input` - The user's input to be processed by the AI.
/// * `model` - The OpenAI model to be used for generating the response.
/// * `api_key` - The OpenAI API key for authentication.
/// * `user_channel_key` - A tuple representing the user and channel IDs.
/// * `chat_history` - The chat history to be used as context for generating the AI response.
///
pub async fn generate_ai_response(
  prompt: &str,
  model: &str,
  api_key: &str,
  _user_channel_key: (UserId, ChannelId),
  chat_history: String,
) -> Option<String> {
  let client = reqwest::Client::new();

  let url = format!("https://api.openai.com/v1/chat/completions");

  let last_message_id = chat_history;

  let params = json!({
    "model": model,
    "messages": [{"role": "system", "content": "You are a helpful assistant."}, {"role": "user", "content": &last_message_id}, {"role": "user", "content": prompt}],
    "max_tokens": 100,
    "temperature": 0.5,
    // "n": 1,
    // "stop": ["/n"]
  });

  let response = client
    .post(url)
    .header("Authorization", format!("Bearer {}", api_key))
    .header("Content-Type", "application/json")
    .body(params.to_string())
    .send()
    .await;

  // Check if the response was successful
  match response {
    Ok(response) => {
      if response.status().is_success() {
        let response_value: Result<serde_json::Value, _> = response.json().await;

        match response_value {
          Ok(value) => {
            // Deserialize the Value into the ApiResponse struct
            let ai_response: Result<ApiResponse, _> = serde_json::from_value(value.clone());

            match ai_response {
              Ok(api_response) => {
                if let Some(choices) = api_response.choices {
                  // Extract the AI response text from the choices and format it
                  let response_text = choices[0].message.content.trim().replace('\n', " ");
                  if response_text.is_empty() {
                    debug!("AI generated an empty response");
                    None
                  } else {
                    info!("AI generated response: {}", response_text);
                    Some(response_text)
                  }
                } else {
                  debug!("API response does not contain 'choices' field");
                  None
                }
              }
              Err(err) => {
                error!("Error deserializing API response: {:?}", err);
                None
              }
            }
          }
          Err(err) => {
            error!("Error deserializing API response into Value: {:?}", err);
            None
          }
        }
      } else {
        // If the API request failed, print the status, headers, and response text for debugging purposes
        debug!("API request failed with status: {}", response.status());
        debug!("API request failed with headers: {:?}", response.headers());
        let response_text = response
          .text()
          .await
          .unwrap_or_else(|_| "Failed to read response text".to_string());
        debug!("API request failed with response: {}", response_text);
        None
      }
    }
    Err(err) => {
      error!("Error sending API request: {:?}", err);
      None
    }
  }
}

/// Registers the application commands (slash commands) with Discord.
///
/// # Arguments
///
/// * `http` - A reference to the `Http` instance for making requests to Discord API.
///
pub async fn register_application_commands(http: &Http) -> Result<(), Box<dyn std::error::Error>> {
  let commands = http.get_global_application_commands().await?;

  let commands_to_register = vec![
    (
      "chat",
      "Your message to the AI",
      Some(CommandOptionType::String),
    ),
    ("reset", "Reset the chat history", None),
    ("private", "Set the chat privacy to private", None),
    ("public", "Set the chat privacy to public", None),
  ];

  // Check if the commands already exist
  for (name, description, option_type) in commands_to_register {
    let command_exists = commands.iter().any(|c| c.name == *name);

    if !command_exists {
      let command_result = Command::create_global_application_command(http, |command| {
        command.name(name).description(description);
        if let Some(options) = option_type {
          command.create_option(|option| {
            option
              .name(name)
              .description(description)
              .kind(options)
              .required(true)
          });
        }
        command
      })
      .await;

      match command_result {
        Ok(command) => {
          debug!("Successfully registered application command: {:?}", command);
        }
        Err(e) => {
          error!("Error registering application command {}: {:?}", name, e);
        }
      }
    } else {
      debug!("Command {} already exists, skipping...", name);
    }
  }

  debug!(
    "Successfully registered application commands: {:#?}",
    commands
  );

  Ok(())
}
