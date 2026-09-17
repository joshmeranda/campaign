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

	// todo: this bit is messy

	let result = App::new(character_path, status_path, notes_path);

	if let Err(err) = result {
		println!("{}", err.to_string());
		return Ok(())
	}

	let mut app = result.expect("");

	ratatui::run(|terminal| {
		if let Err(err) = app.run(terminal) {
			println!("failed to run app: {}", err.to_string())
		};
	});

	Ok(())
}
