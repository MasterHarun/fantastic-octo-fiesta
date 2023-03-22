use sensible_env_logger::try_init_custom_env_and_builder;
use serenity::{
  async_trait,
  http::Http,
  model::{
    gateway::Ready,
    id::{ChannelId, UserId},
    prelude::{
      command::{Command, CommandOptionType},
      interaction::{Interaction, InteractionResponseType},
    },
  },
  prelude::*,
};

use std::sync::{Arc, Mutex};
use std::{collections::HashMap, env};

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
      // Extract the content from the command's options
      let content = command
        .data
        .options
        .get(0)
        .and_then(|opt| opt.value.as_ref())
        .and_then(|val| val.as_str());

      if let Some(content) = content {
        // Get the user and their user# who sent the message
        let user = &command.user.name;
        let user_id = &command.user.discriminator;

        // Log the user and their user# along with the command input
        info!("User {}#{}: {}", user, user_id, content);

        let model = "gpt-3.5-turbo";
        let api_key = env::var("OPENAI_API_KEY").expect("Expected OPENAI_API_KEY to be set");
        let user_channel_key = (command.user.id, command.channel_id);

        // Get the chat history for the user and channel
        let chat_history = {
          let mut chat_histories = self.chat_histories.lock().unwrap();
          chat_histories
            .entry(user_channel_key)
            .or_insert_with(String::new)
            .clone()
        };

        // Generate the AI response using the chat history and content
        let result =
          generate_ai_response(content, model, &api_key, user_channel_key, chat_history).await;

        // Update the chat history with the AI response
        if let Some(ref ai_response) = result {
          let mut chat_histories = self.chat_histories.lock().unwrap();
          let history = chat_histories.get_mut(&user_channel_key).unwrap();
          history.push_str(" ");
          history.push_str(ai_response);
        }

        // Acknowledge the interaction first
        if let Err(why) = command
          .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
          })
          .await
        {
          error!("Error acknowledging interaction: {:?}", why);
        }

        // Then send the follow-up message with the AI response
        match result {
          Some(ai_response) => {
            if let Err(why) = command
              .create_followup_message(&ctx.http, |message| {
                message.content(&ai_response);
                message
              })
              .await
            {
              error!("Error sending response: {:?}", why);
              // Debugging: Log the command, interaction token, and application ID
              debug!(
                "Debugging info: command: {:?}, interaction token: {:?}, application ID: {:?}",
                command,
                command.token,
                ctx.http.application_id(),
              );
            }
          }
          None => {
            if let Err(why) = command
              .create_followup_message(&ctx.http, |message| {
                message.content("Error: AI response could not be generated.");
                message
              })
              .await
            {
              error!("Error sending error message: {:?}", why);
              // Debugging: Log the command, interaction token, and application ID
              debug!(
                "Debugging info: command: {:?}, interaction token: {:?}, application ID: {:?}",
                command,
                command.token,
                ctx.http.application_id(),
              );
            }
          }
        }
      } else {
        // If the command is missing input text, send an error message
        if let Err(why) = command
          .create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource);
            response.interaction_response_data(|message| {
              message.content("Error: Command missing input text.");
              message
            })
          })
          .await
        {
          error!("Error sending error message: {:?}", why);
        }
      }
    }
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
  let commands_to_register = vec![("ask", "Your message to the AI", CommandOptionType::String)];

  // Check if the commands already exist
  for (name, description, options) in commands_to_register {
    let command_exists = commands.iter().any(|c| c.name == *name);

    // If the command doesn't exist, create it and add it as a global command
    if !command_exists {
      let command_result = Command::create_global_application_command(http, |command| {
        command
          .name(name)
          .description(description)
          .create_option(|option| {
            option
              .name(name)
              .description(description)
              .kind(options)
              .required(true)
          })
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
