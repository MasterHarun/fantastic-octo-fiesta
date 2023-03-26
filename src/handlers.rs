use std::sync::{Arc, Mutex};
use std::{collections::HashMap};

use serenity::{
	async_trait,
	http::Http,
	model::{
		gateway::Ready,
		id::{ChannelId, UserId},
		prelude::{
			interaction::{
				Interaction,
			}
		},
	},
	prelude::{
		EventHandler,
		Context,
	}
};

use crate::commands::*;
use crate::utils::{register_application_commands, acknowledge_interaction};

pub struct Handler {
  chat_histories: Arc<Mutex<HashMap<(UserId, ChannelId), String>>>,
  chat_privacy: Arc<Mutex<HashMap<UserId, bool>>>,
  http: Arc<Http>,
}

impl Handler {
  // Create a new Handler with the given Http instance
  pub fn new(http: Arc<Http>) -> Self {
    Self {
      chat_histories: Arc::new(Mutex::new(HashMap::new())),
      chat_privacy: Arc::new(Mutex::new(HashMap::new())),
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

      // Acknowledge the interaction
      let interaction_data = match acknowledge_interaction(&command, &ctx, ephemeral).await {
        Ok(data) => data,
        Err(_) => return,
      };

      match command.data.name.as_str() {
        "chat" => chat_command(&self.chat_histories, &ctx, &command, is_private, &interaction_data).await,
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
