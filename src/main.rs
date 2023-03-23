use sensible_env_logger::try_init_custom_env_and_builder;
use serenity::{
  async_trait,
  http::Http,
  model::{
    application::interaction::application_command::ApplicationCommandInteraction,
    gateway::Ready,
    id::{ChannelId, UserId},
    prelude::{
      command::{Command, CommandOptionType},
      interaction::{Interaction, InteractionResponseType},
    },
  },
  prelude::*,
};
use unicode_segmentation::UnicodeSegmentation;

use std::sync::{Arc, Mutex};
use std::{collections::HashMap, env};
use tokio::time::{timeout, Duration};

use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::json;

extern crate sensible_env_logger;
#[macro_use]
extern crate log;

// Define a handler struct with a Mutex to store the chat history
struct Handler {
  chat_histories: Arc<Mutex<HashMap<(UserId, ChannelId), String>>>,
  http: Arc<Http>,
}

impl Handler {
  // Create a new Handler with the given Http instance
  fn new(http: Arc<Http>) -> Self {
    Self {
      chat_histories: Arc::new(Mutex::new(HashMap::new())),
      http,
    }
  }
}

#[async_trait]
impl EventHandler for Handler {
  // Event handler for when the bot is ready
  async fn ready(&self, _: Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);

    // Register application commands for the bot
    if let Err(e) = register_application_commands(&self.http).await {
      error!("Error registering application commands: {:?}", e);
    }
  }

  // Event handler for when an interaction is created
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
      // Acknowledge the interaction
      if let Err(_) = acknowledge_interaction(&command, &ctx).await {
        return;
      }

      let user_id = command.user.id;
      let channel_id = command.channel_id;
      let user_name = command.user.name.clone();
      let user_channel_key = (user_id, channel_id);

      match command.data.name.as_str() {
        "chat" => {
          let input = command
            .data
            .options
            .get(0)
            .and_then(|opt| opt.value.as_ref())
            .and_then(|value| value.as_str())
            .unwrap_or("");

          // Log the user's input with their name and ID
          info!(
            "User {}#{}: {}",
            user_name, command.user.discriminator, input
          );

          // Generate AI response here
          let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found");
          let model = "gpt-3.5-turbo";

          let chat_history = {
            let mut chat_histories = self.chat_histories.lock().unwrap();
            chat_histories
              .entry(user_channel_key)
              .or_insert_with(String::new)
              .clone()
          };

          let ai_response =
            generate_ai_response(input, model, &api_key, user_channel_key, chat_history).await;

          // Send the follow-up message
          if let Some(response) = ai_response.clone() {
            if let Err(_) = create_followup_message(&ctx, &command, response).await {
              return;
            }
          } else {
            error!("Error: AI response could not be generated.");
            return;
          }

          // Update the chat history with the user's input and the AI's response
          {
            let mut chat_histories = self.chat_histories.lock().unwrap();
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

        "reset" => {
          // Remove the chat history for the user who called the command
          {
            let mut chat_histories = self.chat_histories.lock().unwrap();
            chat_histories.remove(&(user_id, channel_id));
          }

          // Send a follow-up message to indicate the chat history has been reset
          let reset_message = "Chat history has been reset.".to_string();

          if let Err(_) = create_followup_message(&ctx, &command, reset_message).await {
            return;
          }
        }
        _ => {
          error!("Unknown command: {}", command.data.name);
        }
      }
    }
  }
}

async fn create_followup_message(
  ctx: &Context,
  command: &ApplicationCommandInteraction,
  content: String,
) -> Result<(), ()> {
  match command
    .create_followup_message(&ctx.http, |message| {
      message.content(content);
      message
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

async fn acknowledge_interaction(
  command: &ApplicationCommandInteraction,
  ctx: &Context,
) -> Result<(), ()> {
  match timeout(
    Duration::from_millis(2500),
    command.create_interaction_response(&ctx.http, |response| {
      response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
    }),
  )
  .await
  {
    Ok(Ok(_)) => {
      debug!("Acknowledged the interaction");
      Ok(())
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

fn truncate_chat_history(chat_history: &mut String, max_tokens: usize) {
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

// Deserialize the API response into these structs
#[derive(Debug, Deserialize)]
struct ApiResponse {
  choices: Option<Vec<Choice>>,
}

#[derive(Debug, Deserialize)]
struct Choice {
  message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
  content: String,
}

// Generate an AI response based on the given input and chat history
async fn generate_ai_response(
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
async fn register_application_commands(http: &Http) -> Result<(), Box<dyn std::error::Error>> {
  // Get the existing gloval application commands
  let commands = http.get_global_application_commands().await?;

  // Define the commands to register, along with their name, description, and options
  let commands_to_register = vec![
    ("chat", "Your message to the AI", Some(CommandOptionType::String)),
    (
      "reset",
      "Reset the chat history",
      None,
    ),
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

#[tokio::main]
async fn main() {
  dotenv().ok();
  info!("running");
  // Get env vars
  let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
  let application_id =
    env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID not found");

  // Initialize the logger
  let _ = try_init_custom_env_and_builder(
    &env::var("RUST_LOG").expect("RUST_LOG not found"),
    &env::var("GLOBAL_LOG_LEVEL").expect("GLOBAL_LOG_LEVEL not found"),
    env!("CARGO_PKG_NAME"),
    module_path!(),
    sensible_env_logger::pretty::formatted_timed_builder,
  );

  // Set the gateway intents to receive guild messages and message content
  let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

  // Create an HTTP client and Discord bot client
  let http = Arc::new(Http::new_with_application_id(
    &token,
    application_id.parse::<u64>().unwrap(),
  ));
  let mut client = serenity::Client::builder(&token, intents)
    .intents(intents)
    .event_handler(Handler::new(Arc::clone(&http)))
    .await
    .expect("Error creating client");

  // Start the Discord bot client and log any errors
  if let Err(why) = client.start().await {
    error!("Client error: {:?}", why);
  }
}
