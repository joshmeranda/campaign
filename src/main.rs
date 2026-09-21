mod types;
mod character;
mod error;

use character::App;
use std::error::Error;
use std::fs;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::error::AppError;
use crate::types::{Character, Status};

const CHARACTER_FILENAME: &str = "character.yaml";
const STATUS_FILENAME: &str = "status.yaml";
const NOTES_FILENAME: &str = "notes.txt";

#[derive(Subcommand)]
enum CharacterSubCommand {
	// Create a new characte directory with 
	New {
		// The name of the character to create
		#[arg(default_value = "new-character")]
		name: String,
	},

	View {
		dir: PathBuf
	},
}

#[derive(Subcommand)]
enum MainSubcommand {
	// Perform operations on a character
	Character {
		#[command(subcommand)]
		command: CharacterSubCommand
	},
} 

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: MainSubcommand,
}

fn do_character(character_sub_command: CharacterSubCommand) -> Result<(), AppError> {
	match character_sub_command {
		CharacterSubCommand::New { name } => {
			let dir = PathBuf::from(&name);
			fs::create_dir(&dir)?;

			let notes_file = dir.join("notes.txt");
			fs::write(notes_file, format!("# {}
Thanks for using Campaign!

As you are creating you character be sure to flesh out the following characterisitics for your character:
- age:
- appearence:
- history:
- motivations:
", &name))?;

			let character_file = dir.join("character.yaml");
			let data = serde_yaml::to_string(&Character::with_name(name))?;
			fs::write(character_file, data)?;

			let status_file = dir.join("status.yaml");
			let data = serde_yaml::to_string(&Status::default())?;
			fs::write(status_file, data)?;
		},
		CharacterSubCommand::View { dir } => {
			let character_path = dir.join(CHARACTER_FILENAME);
			let status_path = dir.join(STATUS_FILENAME);
			let notes_path = dir.join(NOTES_FILENAME);
				let mut app = App::new(character_path, status_path, notes_path)?;

				ratatui::run(|terminal| {
					if let Err(err) = app.run(terminal) {
						println!("failed to run app: {}", err.to_string())
					};
				});
		}
	}

	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	let cli = Cli::parse();

	match cli.command {
		MainSubcommand::Character { command } =>  do_character(command)?,
	}

	Ok(())
}
