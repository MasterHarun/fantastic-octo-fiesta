# **RustGPT-Discord**
RustGPT-Discord is a feature-rich bot built using the Rust programming language and the Serenity library. This bot provides various commands and functionalities, including AI chat integration and privacy settings for messages.

## **Prerequisites**
---
Before setting up the bot, ensure you have the following:

[Rust](https://www.rust-lang.org/tools/install) installed on your system.
A Discord account and [Discord Developer Portal](https://discord.com/developers/applications) access.
[OpenAI](https://beta.openai.com/signup/) account for API access.

## **Using the Release Binary**
---
If you prefer to use a pre-built binary of the RustGPT-Discord bot, follow these steps:

### **1**. **Download the binary**
Navigate to the [Releases](https://github.com/MasterHarun/rustgpt-discord/release) page of the RustGPT-Discord repository.

Download the latest release binary for your operating system.

### **2**. **Set up environment variables**
As explained in the "Set up environment variables" section above, you can choose to use a .env file, command-line flags, or set environment variables to configure the bot.

### **3**. **Run the binary**
Open a terminal or command prompt and navigate to the folder where you downloaded the release binary.

Run the binary with the necessary command-line flags or ensure the environment variables are set:

```sh
./rustgpt-discord -t your_token -a your_app_id -o your_api_key
```
Replace `your_token`, `your_app_id`, and `your_api_key` with the appropriate values.

The bot should now be running, and it will display a "Connected" message in the terminal.

Remember to follow the instructions in the ["Invite the bot to your Discord server"](#5-invite-the-bot-to-your-discord-server) section to invite the bot to your server if you haven't already done so.

## **Setup**
---
Follow the steps below to set up the bot:

### **1**. **Create a new Discord application**
Go to the [Discord Developer Portal](https://discord.com/developers/applications) and log in with your Discord account.
Click "New Application" and provide a name for your bot.

In the "General Information" tab of your Discord application settings, you'll find the "Application ID". Save this value as you will need it later.

Navigate to the "Bot" tab on the left sidebar, and click "Add Bot".

Copy the bot token by clicking "Copy" under the "TOKEN" section. You will need this token later.

Scroll down to the "Privileged Gateway Intents" section, and enable the following intents:

- `MESSAGE CONTENT INTENT`

This allows the bot to receive messages.

Click the "Save Changes" button.
### **2**. **Clone the repository and install dependencies**
Copy code:
```sh
git clone https://github.com/MasterHarun/rustgpt-discord.git
cd rustgpt-discord
```

### **3**. **Set up environment variables**
Create a `.env` file in the root directory of the project and set the necessary environment variable. Alternatively, you can set these variables directly in your system or pass them as command-line flags when running the bot.

- #### **Using a `.env` file**

```sh
DISCORD_TOKEN=<your_discord_bot_token>
DISCORD_APP_ID=<your_discord_app_id>
OPENAI_API_KEY=<your_openAI_api_key>
```

- #### **Using Command-line flags** 
```sh
./rustgpt-discord --token your_token --discord-app-id your_app_id --openai-api-key your_openai_api_key
```

- #### **Using Env vars**
```sh
export DISCORD_TOKEN=your_token
export DISCORD_APP_ID=your_app_id
export OPENAI_API_KEY=your_openai_key
```

>Replace `<your_discord_bot_token>` with the bot token you copied earlier, and `<your_openAI_api_key>` with your OpenAI API key.

### **4**. **Building and Running the Binary**
To build the binary, run the following command in the project root:

```sh
cargo build --release
```

The binary will be created in the target/release folder. To run the binary, navigate to the target/release folder and execute the binary:
```sh
cd target/release
./rustgpt-discord
```
If you want to use command-line flags, provide them when running the binary:

```sh
./rustgpt-discord --token your_token --discord-app-id your_app_id --openai-api-key your_openai_api_key
```
The bot should now be running, and it will display a "Connected" message in the terminal.

### **5**. **Invite the bot to your Discord server**
Go to the [Discord Developer Portal](https://discord.com/developers/applications) and select your bot.
Navigate to the "OAuth2" tab on the left sidebar.
Under the "Scopes" section, select the "bot" checkbox.
Under the "Bot Permissions" section, select the desired permissions for your bot.
Copy the generated URL from the "Scopes" section.
Open the URL in a new browser tab, and follow the prompts to invite the bot to your server.
The bot should now appear in your server, and you can start using its commands and features.

## **Commands**
---
Here is a list of available commands:

**/chat**: Chat with the AI using OpenAI's GPT.

**/reset**: Reset the chat context with the AI.

**/private**: Set chat privacy mode to "Private," making the AI responses visible only to the command issuer.

**/public**: Set chat privacy mode to "Public," making the AI responses visible to all server members.

## **Features**
---
AI chat integration using OpenAI's GPT.
Chat privacy settings allowing for private or public AI responses.
Extensible command system for adding new commands easily.

## **Contributing**
---
Contributions are welcome! If you'd like to contribute, please follow these steps:

Fork the repository.
Create a new branch for your changes.
Make your changes and commit them with a descriptive commit message.
Push your changes to your fork.
Create a pull request.

## **License**
This project is licensed under the **MIT License**.