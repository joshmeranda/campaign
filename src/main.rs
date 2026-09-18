mod types;
mod character;
mod error;

use character::App;
use std::error::Error;

const CHARACTER_FILENAME: &str = "character.yaml";
const STATUS_FILENAME: &str = "status.yaml";
const NOTES_FILENAME: &str = "notes.txt";

fn main() -> Result<(), Box<dyn Error>> {
	let path = std::path::Path::new("./.character");
	let character_path = path.join(CHARACTER_FILENAME);
	let status_path = path.join(STATUS_FILENAME);
	let notes_path = path.join(NOTES_FILENAME);

	let mut app = App::new(character_path, status_path, notes_path)?;

	ratatui::run(|terminal| {
		if let Err(err) = app.run(terminal) {
			println!("failed to run app: {}", err.to_string())
		};
	});

	Ok(())
}
