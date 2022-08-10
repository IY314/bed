use std::{collections::BTreeMap, io::Write};

use rustyline::error::ReadlineError;
use text_io::try_read;

struct State<'a> {
    buffer: &'a mut BTreeMap<usize, String>,
}

fn process_input(state: &mut State, input: String) {
    if let Ok(ln) = {
        let ln_res: Result<usize, _> = try_read!("{}", input.bytes());
        ln_res
    } {
        state.buffer.insert(ln, input);
    } else {
        if input == "r" {
            let mut new_buf: BTreeMap<usize, String> = BTreeMap::new();

            for (i, line) in state.buffer.clone().values().enumerate() {
                let mut new_line = line.to_owned();

                let ln = (i + 1) * 10;

                // Every line edited will always begin with a number.
                let space_idx = new_line.find(|c: char| c.is_whitespace()).unwrap();
                new_line = new_line[space_idx..].to_string();
                new_line = ln.to_string() + &new_line;
                new_buf.insert(ln, new_line);
            }

            state.buffer.clear();
            state.buffer.append(&mut new_buf);
        } else if let Ok(w) = {
            let w_res: Result<String, _> = try_read!("w {}", input.bytes());
            w_res
        } {
            let mut file = std::fs::File::create(w).expect("Could not create file");
            file.write_all(
                state
                    .buffer
                    .values()
                    .cloned()
                    .collect::<Vec<String>>()
                    .join("\n")
                    .as_bytes(),
            )
            .expect("Could not write to file");
        } else if input == "p" {
            for line in state.buffer.values().cloned().collect::<Vec<String>>() {
                println!("{}", line);
            }
        } else {
            println!("Unknown command");
        }
    }
}

fn main() -> rustyline::Result<()> {
    // `()` can be used when no completer is required
    let hist_file = dirs::home_dir()
        .expect("Could not get home directory")
        .join(".bed_history")
        .into_os_string()
        .into_string()
        .expect("Could not convert path into string");
    let mut rl = rustyline::Editor::<()>::new()?;
    if rl.load_history(hist_file.as_str()).is_err() {
        println!("No previous history.");
    }
    let mut state = State {
        buffer: &mut BTreeMap::new(),
    };

    loop {
        let readline = rl.readline(":");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                process_input(&mut state, line);
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
