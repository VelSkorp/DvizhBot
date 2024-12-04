# DvizhBot

A feature-rich Telegram bot implemented in Rust, designed to manage events, interact with users, and provide various utility functionalities. This bot leverages modular design for scalability and maintainability.

---

## Features

- **Command Parsing**: Supports multi-word arguments for commands like `/addevent [title] [date] [location] [description]`.
- **Event Management**: Handles event creation, storage, and retrieval.
- **Localization**: Provides multi-language support, allowing users to select their preferred language.
- **Zodiac Sign Interaction**: Lets users select and interact with zodiac signs.
- **Database Integration**: Stores user data, language preferences, and events in SQLite using a clear schema.
- **Error Handling**: Implements structured error management for robust operation.

---

## Project Structure

### Core Modules

- **message_handler.rs**: Handles incoming messages and delegates commands to appropriate handlers.
- **messaging.rs**: Contains utilities for sending messages, inline keyboards, and replies.
- **msg_type_utils.rs**: Provides utilities for defining and managing different message types.
- **tg_bot.rs**: Main bot logic, manages updates and integrates various components.
- **tg_objects.rs**: Defines core Telegram objects like `Update`, `Message`, and `CallbackQuery`.
- **tg_utils.rs**: Contains helper functions for interacting with the Telegram API.
- **command_utils.rs**: Processes and parses commands for extracting arguments and executing actions.
- **commands.rs**: Contains implementations for specific bot commands like `/start` and `/addevent`.
- **events.rs**: Manages event-related functionalities, such as creation and retrieval.
- **language_utils.rs**: Handles language-related operations, such as language detection and preference updates.

---

## Setup

### Prerequisites

- **Rust**: Ensure you have Rust installed on your system. Visit [rust-lang.org](https://www.rust-lang.org/) for installation instructions.
- **SQLite**: The bot requires an SQLite database to store user and event data.

### Installation

1. Clone the repository:
    ```bash
    git clone https://github.com/your-repo/telegram-bot.git
    cd telegram-bot
    ```

2. Install dependencies:
    ```bash
    cargo build
    ```

3. Configure the bot token in the config.json file located in the project folder.

4. Run the bot:
    ```bash
    cargo run
    ```

## Usage

### Available Commands

- `/start`: Registers a new user and sends a language selection keyboard.
- `/addevent [title] [date] [location] [description]`: Adds a new event. Supports multi-word arguments for all fields.
- `/events`: Lists all upcoming events.
- `/setlanguage [language]`: Sets the preferred language for the user.
- `/zodiac`: Allows users to select and interact with zodiac signs.

### Callback Queries

- Language Selection: Inline buttons allow users to choose their preferred language (English, Russian, Polish).
- Zodiac Signs: Inline buttons enable users to select and interact with zodiac signs.

## Contributing

Contributions are welcome! Please submit issues or pull requests with any improvements, bug fixes, or new features.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.