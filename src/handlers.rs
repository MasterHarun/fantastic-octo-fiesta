//! Define `Handler` struct and implement the `EventHandler` trait for it
//!
//! ## Interaction handling
//!
//! Implement the `interaction_create` method to handle incoming interactions
//! and delegate command handling to the appropriate functions from the `commands` module.
//!

use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

use serenity::{
  async_trait,
  http::Http,
  model::{
    gateway::Ready,
    id::{UserId},
    prelude::interaction::Interaction,
  },
  prelude::{Context, EventHandler},
};

use crate::structures::ConfigStruct;
use crate::users::*;
use crate::utils::{acknowledge_interaction, register_application_commands};
use crate::commands::*;


#[derive(Clone)]
pub struct HandlerStruct {
	users: Arc<Mutex<FxHashMap<UserId, User>>>,
  personas: Arc<Mutex<Vec<Personality>>>,
  config: Arc<ConfigStruct>,
}
impl HandlerStruct {
  pub fn new(config: Arc<ConfigStruct>) -> Self {
    Self {
      users: Arc::new(Mutex::new(FxHashMap::default())),
      personas: Arc::new(Mutex::new(Vec::new())),
      config,
    }
  }

  pub fn user_exists(&self, user_id: UserId) -> bool {
    self.users.lock().unwrap().contains_key(&user_id)
  }

  pub fn add_user(&self, user_id: UserId) {
    self
      .users
      .lock()
      .unwrap()
      .insert(user_id, User::new(user_id));
  }
	pub fn modify_user<F>(&self, user_id: UserId, modify: F) -> Result<(), String>
	where
			F: FnOnce(&mut User) + Send,
	{
			let mut users = self.users.lock().unwrap();
			if let Some(user) = users.get_mut(&user_id) {
					modify(user);
					Ok(())
			} else {
					Err(String::from("User not found"))
			}
	}
  pub fn with_user<F, R>(&self, user_id: UserId, f: F) -> Option<R>
	where
		F: FnOnce(&User) -> R,
	{
		let users = self.users.lock().unwrap();
		// if let Some(user) = users.get(&user_id) {
		// 	Some(f(user))
		// } else {
		// 	None
		// }
		// This is equivalent to the above
		users.get(&user_id).map(f)
	}
	pub fn modify_personas<F>(&self, modify: F) -> Result<(), String>
	where
			F: FnOnce(&mut Vec<Personality>) + Send,
	{
			let mut personas = self.personas.lock().unwrap();
			modify(&mut personas);
			Ok(())
	}
	pub fn set_default_personas(&self) {
		let mut personas = match self.personas.lock() {
			Ok(p) => p,
			Err(e) => {
					eprintln!("Error acquiring lock: {}", e);
					return;
			}
	};
	let personas_json = match std::fs::read_to_string("personas.json") {
			Ok(json) => json,
			Err(e) => {
					eprintln!("Error reading file: {}", e);
					return;
			}
	};
	let personas_vec: Vec<Personality> = match serde_json::from_str(&personas_json) {
			Ok(vec) => vec,
			Err(e) => {
					eprintln!("Error parsing json: {}", e);
					return;
			}
	};
	for persona in personas_vec {
			personas.push(persona);
	}

	}

	pub fn get_personas(&self) -> Vec<Personality> {
		match self.personas.lock() {
			Ok(personas) => personas.clone(),
			Err(e) => {
				eprintln!("Error while getting personas: {:?}", e);
				Vec::new()
			}
		}
	}
	
  pub fn get_config(&self) -> Arc<ConfigStruct> {
    self.config.clone()
  }
	
}

#[async_trait]
impl EventHandler for HandlerStruct {
  async fn ready(&self, _: Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);
    let http = Arc::new(Http::new_with_application_id(
      &self.config.discord_token,
      self.config.app_id.parse::<u64>().unwrap(),
    ));
		// set the default personas for the bot
		self.set_default_personas();
    if let Err(e) = register_application_commands(&http).await {
      error!("Error registering application commands: {:?}", e);
    }
  }

  ///
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
      if !self.user_exists(user_id) {
        self.add_user(user_id);
      }

			let total_tokens = self.with_user(user_id, |user| user.with_usage(|usage| usage.get_total_tokens())).unwrap();
			debug!("Total tokens: {}", total_tokens);
			let chat_privacy = self.with_user(user_id, |user| user.with_settings(|settings| settings.get_chat_privacy())).unwrap();
			let ephemeral = match command.data.name.as_str() {
				"private" | "public" => true,
				_ => chat_privacy
				//  chat_privacy == ChatPrivacy::Private
			};
					
			acknowledge_interaction(&command, &ctx, ephemeral).await;

      match command.data.name.as_str() {
        "chat" => chat_command(self, &ctx, &command).await,
        "prompt" => {
          todo!()
        }
        "personality" => personality_command(self, &ctx, &command).await,
        "reset" => reset_command(self, &ctx, &command).await,
        "private" => private_command(self, &ctx, &command).await,
        "public" => public_command(self, &ctx, &command).await,
				"addpersonality" => add_personality_command(self, &ctx, &command).await,
        _ => {
          error!("Unknown command: {}", command.data.name);
        }
      }	
    }
  }
}
