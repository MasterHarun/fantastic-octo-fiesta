 //! Define `Handler` struct and implement the `EventHandler` trait for it
  //!
  //! ## Interaction handling
  //!
  //! Implement the `interaction_create` method to handle incoming interactions
  //! and delegate command handling to the appropriate functions from the `commands` module.
  //!

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serenity::{
  async_trait,
  http::Http,
  model::{
    gateway::Ready,
    id::{ChannelId, UserId},
    prelude::interaction::Interaction,
  },
  prelude::{Context, EventHandler},
};

use crate::commands::*;
use crate::structures::Config;
use crate::utils::{acknowledge_interaction, register_application_commands};

pub struct Handler {
  chat_histories: Arc<Mutex<HashMap<(UserId, ChannelId), String>>>,
  chat_privacy: Arc<Mutex<HashMap<UserId, bool>>>,
	config: Arc<Config>,
}

impl Handler {
  pub fn new(config: Arc<Config>) -> Self {
    Self {
      chat_histories: Arc::new(Mutex::new(HashMap::new())),
      chat_privacy: Arc::new(Mutex::new(HashMap::new())),
			config,
    }
  }
}

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _: Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);
		let http = Arc::new(Http::new_with_application_id(
			&self.config.discord_token,
			self.config.app_id.parse::<u64>().unwrap(),
		));
    if let Err(e) = register_application_commands(&http).await {
      error!("Error registering application commands: {:?}", e);
    }
  }

/// Handles interaction events
///
/// Processes the user's interaction with the bot and executes the corresponding command.
///
/// # Arguments
///
/// * `ctx` - The Serenity Context for the event
/// * `interaction` - The Interaction data
/// 
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
      let user_id = command.user.id;
      let is_private = *self
        .chat_privacy
        .lock()
        .unwrap()
        .get(&user_id)
        .unwrap_or(&false);

      let ephemeral = match command.data.name.as_str() {
        "private" | "public" => true,
        _ => is_private,
      };

      let interaction_data = match acknowledge_interaction(&command, &ctx, ephemeral).await {
        Ok(data) => data,
        Err(_) => return,
      };

      match command.data.name.as_str() {
        "chat" => {
          chat_command(
            &self.chat_histories,
            &ctx,
            &command,
            is_private,
            &interaction_data,
						&self.config.api_key,
          )
          .await
        }
        "reset" => reset_command(&self.chat_histories, &ctx, &command, is_private).await,
        "private" => private_command(&self.chat_privacy, &ctx, &command, &interaction_data).await,
        "public" => public_command(&self.chat_privacy, &ctx, &command, &interaction_data).await,
        _ => {
          error!("Unknown command: {}", command.data.name);
        }
      }
    }
  }
}
