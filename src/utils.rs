use serde_json::json;
use serenity::{model::prelude::{interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType}, UserId, ChannelId, command::{CommandOptionType, Command}}, prelude::Context, http::Http};
use tokio::time::{timeout, Duration};
use unicode_segmentation::UnicodeSegmentation;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap};

use crate::structures::*;

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

pub async fn edit_original_message_or_create_followup(
	ctx: &Context,
	command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
	content: String,
	chat_privacy: bool,
) -> Result<(), ()> {
	let _interaction_id = &interaction_data.interaction_id;
	let response_token = &interaction_data.response_token;

	let message = if chat_privacy {
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
			if let Err(why) = create_followup_message(ctx, command, content, chat_privacy).await {
					error!("Error sending follow-up message: {:?}", why);
					return Err(());
			}
			debug!("Sent a follow-up message");
			return Ok(());
	}
}


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

pub(crate) async fn set_chat_privacy(
	chat_privacy_map: &Arc<Mutex<HashMap<UserId, bool>>>,
	user_id: UserId,
	chat_privacy: bool,
	ctx: &Context,
	command: &ApplicationCommandInteraction,
	interaction_data: &InteractionData,
) {
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


// Generate an AI response based on the given input and chat history
pub async fn generate_ai_response(
  prompt: &str,
  model: &str,
  api_key: &str,
  _user_channel_key: (UserId, ChannelId),
  chat_history: String,
) -> Option<String> {
  // Create a client for the OpenAI API
  let client = reqwest::Client::new();

  // Set the API URL
  let url = format!("https://api.openai.com/v1/chat/completions");

  let last_message_id = chat_history;

  // Set the parameters for the API call
  let params = json!({
    "model": model,
    "messages": [{"role": "system", "content": "You are a helpful assistant."}, {"role": "user", "content": &last_message_id}, {"role": "user", "content": prompt}],
    "max_tokens": 100,
    "temperature": 0.5,
    // "n": 1,
    // "stop": ["/n"]
  });

  // Send the API request and get the response
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
        // Return None since the request failed
        None
      }
    }
    Err(err) => {
      // If there was an error sending the API request, print the error for debugging purposes
      error!("Error sending API request: {:?}", err);
      // Return None since the request could not be sent
      None
    }
  }
}


// Registers the application commands that the Discord bot can receive.
pub async fn register_application_commands(http: &Http) -> Result<(), Box<dyn std::error::Error>> {
  // Get the existing gloval application commands
  let commands = http.get_global_application_commands().await?;

  // Define the commands to register, along with their name, description, and options
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

    // If the command doesn't exist, create it and add it as a global command
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

      // Log the result of registering the command
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

  // Log the list of registered commands
  debug!(
    "Successfully registered application commands: {:#?}",
    commands
  );

  Ok(())
}
