pub mod collection_commands;
pub mod crispi_commands;
pub mod deck_commands;
pub mod image_commands;
pub mod settings_commands;
pub mod sidecar_commands;

pub use self::deck_commands::create_deck;
pub use self::settings_commands::{check_for_updates, install_update};

pub use self::deck_commands::test_command;
