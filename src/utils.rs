//! Contains utility functions to support the main functionality of the bot
//!
//! ## Utility functions
//!
//! - `register_application_commands`: Registers application commands with Discord
//! - `generate_ai_response`: Generates an AI response using the OpenAI API
//! - `acknowledge_interaction`: Acknowledges an interaction with Discord
//! - `create_followup_message`: Sends a follow-up message for an interaction
//! - `edit_original_message_or_create_followup`: Edits the original interaction message or creates a follow-up message
//! - `set_chat_privacy`: Sets chat privacy for a user
//! - `get_env_var`: Gets the environment variables from various sources.
//!

use serde_json::json;
use serenity::{
  builder::CreateApplicationCommand,
  http::Http,
  model::{
    prelude::{
      command::{Command, CommandOptionType},
      interaction::{application_command::ApplicationCommandInteraction, InteractionResponseType},
      ChannelId, UserId,
    },
    Permissions,
  },
  prelude::Context,
};
use tokio::time::{timeout, Duration};

use crate::{handlers::HandlerStruct, structures::*};

/// Creates a follow-up message in response to an application command (slash command).
/// This function checks the chat privacy setting for the user and sends an ephemeral message if the setting is enabled.
///
/// ### Arguments
///
/// * `ctx` - The `Context` for accessing the Discord API.
/// * `command` - The `ApplicationCommandInteraction` that triggered the follow-up message.
/// * `content` - The content of the follow-up message.
/// * `chat_privacy` - A boolean representing the privacy setting of the message.
///
/// ### Returns
///
/// * `Result<(), ()>` - A `Result` containing the result of the operation.
///
/// ### Errors
///
/// * `()` - An error occurred while sending the follow-up message.
///
pub async fn create_followup_message(
  ctx: &Context,
  command: &ApplicationCommandInteraction,
  content: String,
  chat_privacy: &bool,
) -> Result<(), ()> {
  match command
    .create_followup_message(&ctx.http, |message| {
      if *chat_privacy {
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
/// ### Arguments
///
/// * `ctx` - The Serenity Context
/// * `command` - The ApplicationCommandInteraction data
/// * `content` - The content of the message
/// todo: review this function
pub async fn edit_original_message_or_create_followup(
  ctx: &Context,
  command: &ApplicationCommandInteraction,
  content: String,
  chat_privacy: &bool,
) -> Result<(), ()> {
  let _interaction_id = command.id.to_string();
  let response_token = command.token.clone();
  let message = if *chat_privacy {
    serde_json::json!({
        "content": content,
        "flags": 64
    })
  } else {
    serde_json::json!({ "content": content })
  };

  if (ctx
    .http
    .edit_original_interaction_response(&response_token, &message)
    .await)
    .is_ok()
  {
    debug!("Edited the original message");
    Ok(())
  } else {
    if let Err(why) = create_followup_message(ctx, command, content, chat_privacy).await {
      error!("Error sending follow-up message: {:?}", why);
      return Err(());
    }
    debug!("Sent a follow-up message");
    Ok(())
  }
}

// / Acknowledges an interaction
///
/// Sends an acknowledgement response to the interaction and returns the interaction data.
///
/// ### Arguments
///
/// * `command` - The ApplicationCommandInteraction data
/// * `ctx` - The Serenity Context for the command
/// * `ephemeral` - A boolean indicating whether the acknowledgement message should be ephemeral
///
pub async fn acknowledge_interaction(
  command: &ApplicationCommandInteraction,
  ctx: &Context,
  ephemeral: bool,
) {
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
    Ok(_) => debug!("Acknowledged the interaction"),
    Err(_) => error!("Timed out while acknowledging the interaction"),
  }
}

/// Sets chat privacy for a user
///
/// Updates the chat privacy settings for a user and sends a follow-up message to indicate the change.
///
/// ### Arguments
///
/// * `handler` - The HandlerStruct for the bot
/// * `chat_privacy` - A boolean representing the new chat privacy setting
/// * `ctx` - The Serenity Context for the command
/// * `command` - The ApplicationCommandInteraction data
///
pub async fn set_chat_privacy(
  handler: &HandlerStruct,
  chat_privacy: bool,
  ctx: &Context,
  command: &ApplicationCommandInteraction,
) {
  let user_id = command.user.id;

  let chat_privacy = if chat_privacy {
    handler
      .modify_user(user_id, |user| {
        user.settings.set_chat_privacy(true);
      })
      .unwrap_or_else(|_| error!("Error setting chat privacy"));
    true
  } else {
    handler
      .modify_user(user_id, |user| {
        user.modify_settings(|settings| settings.set_chat_privacy(false));
      })
      .unwrap_or_else(|_| error!("Error setting chat privacy"));
    false
  };

  let response = if chat_privacy {
    "Chat privacy set to private.".to_string()
  } else {
    "Chat privacy set to public.".to_string()
  };

  if (edit_original_message_or_create_followup(ctx, command, response, &chat_privacy).await)
    .is_err()
  {
    error!("Error setting chat privacy");
  }
}

/// Generates an AI response using the OpenAI API based on the user input and chat history.
///
/// ### Arguments
///
/// * `handler` - The HandlerStruct for the bot
/// * `prompt` - The user input
/// * `user_channel_key` - A tuple containing the user ID and channel ID
///
/// ### Returns
///
/// * `ApiResponse` - The AI response as an ApiResponse struct.
pub async fn generate_ai_response(
  handler: &HandlerStruct,
  prompt: &str,
  user_channel_key: (UserId, ChannelId),
) -> Result<ApiResponseStruct, ()> {
  let client = reqwest::Client::new();
  let user = handler
    .with_user(user_channel_key.0, |user| user.clone())
    .unwrap();
  let user_settings = user.with_settings(|settings| settings.clone());
  let user_usage = user.with_usage(|usage| usage.clone());

  let model = user_settings.get_model();
  let personality = user_settings.get_personality();

  // todo - review how we handle chat history length
  // ? Only once we reach the token threshold for the model?
  // ? How do we determine token count? - Do we need to implement a tokenizer?
  // ? Should we use summarization techniques once the threshold is reached?
  // ? How do we handle the summarization of the chat history?
  // ? How do we store the summarization of the chat history?
  // ? And what about previous portions of the conversation? Should we store them?
  // !? Maybe this could lead to a Memory bank of sort?
  // !? Maybe we could use the chat history to train a model for the user?
  // todo - Handle code blocks
  // ? Maybe store the code blocks in a separate structure and then use it as reference?
  // ? Store the user and AI code blocks separately?
  // ? How do we update the code blocks?
  // ? maybe keep a limit?
  // ? Potentially prompt the user to specify the more recent code blocks?
  let mut chat_history: Vec<Message> = match user_usage.channel_history.get(&user_channel_key.1) {
    Some(channel_data) => {
      let mut history = Vec::new();
      // since the first message is the system message we set it
      history.push(Message {
        role: "system".to_string(),
        content: personality.prompt.clone(),
      });
      for message in channel_data.chat_history.iter() {
        // // we first add the user message as a Message
        if let Some(user_message) = message.get_user_message() {
          history.push(Message {
            role: "user".to_string(),
            content: user_message.clone(),
          });
        }
        // // then we add the AI message as a Message
        if let Some(ai_message) = message.get_ai_message() {
          history.push(Message {
            role: "assistant".to_string(),
            content: ai_message.clone(),
          });
        }
      }
      history
    }
    None => Vec::new(),
  };
  //now we push the user's message to the history
  chat_history.push(Message {
    role: "user".to_string(),
    content: prompt.to_string(),
  });
  // debug!("personality: {:?}", personality);

  debug!("Chat History: {:?}", chat_history);

  let params = ApiRequestBody {
    model: model.get_name(),
    messages: chat_history,
    max_tokens: 300,
    temperature: 0.5,
    user: user_channel_key.0.to_string(),
  };

  let config = handler.get_config();

  let url = "https://api.openai.com/v1/chat/completions".to_string();

  let response = client
    .post(url)
    .header("Authorization", format!("Bearer {}", config.api_key))
    .header("Content-Type", "application/json")
    .body(json!(params).to_string())
    .send()
    .await;

  // then we return the response
  match response {
    Ok(res) => {
      // debug!("Response: {:?}", res);
      let response = res.json::<ApiResponseStruct>().await;
      match response {
        Ok(res) => {
          debug!("Response: {:?}", res);
          // info!("AI Response: {:?} \nTokens Used: {:?}", res.choices[0], res.usage.total_tokens);
          Ok(res)
        }
        Err(why) => {
          error!("Error parsing response: {:?}", why);
          Err(())
        }
      }
    }
    Err(why) => {
      error!("Error sending request: {:?}", why);
      Err(())
    }
  }
}

/// Registers the application commands (slash commands) with Discord.
///
/// ### Arguments
///
/// * `http` - A reference to the `Http` instance for making requests to Discord API.
///
pub async fn register_application_commands(
  handler: &HandlerStruct,
  http: &Http,
) -> Result<(), Box<dyn std::error::Error>> {
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
    // ("model", "Set the AI model", Some(CommandOptionType::SubCommand)),
    (
      "personality",
      "Set the AI personality",
      Some(CommandOptionType::SubCommand),
    ),
  ];

  let admin_commands = vec![(
    "persona-control",
    "Add or remove a personality",
    Some(CommandOptionType::SubCommand),
  )];

  let commands_to_register = commands_to_register
    .into_iter()
    .map(|(name, description, option_type)| (name, description, option_type, false));
  let admin_commands = admin_commands
    .into_iter()
    .map(|(name, description, option_type)| (name, description, option_type, true));
  let commands_to_register = commands_to_register
    .chain(admin_commands)
    .collect::<Vec<_>>();

  debug!("commands_to_register: {:?}", commands_to_register);
  for (name, description, option_type, is_admin) in commands_to_register {
    let command_exists = commands.iter().any(|c| c.name == *name);

    if !command_exists {
      let command_result = Command::create_global_application_command(http, |command| {
        command.name(name).description(description);

        if is_admin {
          command.default_member_permissions(Permissions::ADMINISTRATOR);
          debug!("command: {:?}", command);
        }
        if let Some(options) = option_type {
          match options {
            CommandOptionType::SubCommand => {
              create_options(handler, name, command);
              debug!("SubcommandGroup: {:?}", command);
            }
            CommandOptionType::String => {
              command.create_option(|option| {
                option
                  .name(name)
                  .description(description)
                  .kind(options)
                  .required(true)
              });
            }
            _ => {}
          }
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
fn create_options<'a>(
  handler: &'a HandlerStruct,
  name: &'a str,
  command: &'a mut CreateApplicationCommand,
) -> &'a mut CreateApplicationCommand {
  match name {
    "personality" => {
      let personalities = handler.get_personas();
      command.create_option(|option| {
        option
          .name("choice")
          .description("Set the AI personality")
          .kind(CommandOptionType::String)
          .required(true);
        for personality in personalities {
          debug!("personality: {:?}", personality.name);
          option.add_string_choice(&personality.name, &personality.name);
        }
        option
      });

      command
    }
    "persona-control" => {
      debug!("persona control");
      //add_personalities
      command.create_option(|option| {
        option
          .name("add")
          .description("Add a new personality")
          .kind(CommandOptionType::SubCommand)
          .create_sub_option(|option| {
            option
              .name("name")
              .description("The name of the new personality")
              .kind(CommandOptionType::String)
              .required(true)
          })
          .create_sub_option(|option| {
            option
              .name("description")
              .description("The description of the new personality")
              .kind(CommandOptionType::String)
              .required(true)
          })
          .create_sub_option(|option| {
            option
              .name("prompt")
              .description("The prompt of the new personality")
              .kind(CommandOptionType::String)
              .required(true)
          })
      });
      //remove_personalities
      command.create_option(|option| {
        option
          .name("remove")
          .description("Remove a personality")
          .kind(CommandOptionType::SubCommand)
          .create_sub_option(|option| {
            option
              .name("name")
              .description("The name of the personality to remove")
              .kind(CommandOptionType::String)
              .required(true);
            for persona in handler.get_personas() {
              option.add_string_choice(&persona.name, &persona.name);
            }
            option
          });

        option
      });
      command
    }
    _ => command,
  }
}
/// Retrieves the value of an environment variable or command-line argument.
///
/// This function will first check if the specified command-line argument is provided.
/// If not, it will look for the environment variable with the given name. Lastely it
/// will look to see if a '.env' file exists. If neither options is found, an error
/// message will be displayed, and the program will exit.
///
/// ### Arguments
///
/// * `var_name` - The name of the environment variable to search for.
/// * `cmd_arg` - The name of the command-line argument to search for.
/// * `matches` - An optional reference to the `clap::ArgMatches` object containing the parsed command-line arguments.
///
pub fn get_env_var(var_name: &str, cmd_arg: &str, matches: Option<&clap::ArgMatches>) -> String {
  if let Some(matches) = matches {
    if let Some(value) = matches.get_one::<String>(cmd_arg) {
      value.to_string();
    }
  }
  if let Ok(value) = std::env::var(var_name) {
    value
  } else if let Ok(value) = dotenvy::var(var_name) {
    value
  } else {
    eprintln!("{} not found in command-line arguments, environment variables, or the dotenv file. Please set it up properly.", var_name);
    std::process::exit(1);
  }
}
