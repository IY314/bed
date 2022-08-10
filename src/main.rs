use rustyline::{error::ReadlineError, Editor, Result};

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let hist_file = dirs::home_dir()
        .expect("Could not get home directory")
        .join(".bed_history")
        .into_os_string()
        .into_string()
        .expect("Could not convert path into string");
    let mut rl = Editor::<()>::new()?;
    if rl.load_history(hist_file.as_str()).is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("Received interrupt");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Received EOF");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(hist_file.as_str())
}
