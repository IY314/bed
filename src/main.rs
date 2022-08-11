extern crate alloc;

use alloc::borrow::Cow;

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    io::Write,
    str::FromStr,
};

use rustyline::{
    completion::Completer,
    config::Configurer,
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    history::SearchDirection,
    validate::{ValidationContext, ValidationResult, Validator},
    Context,
};

use ansi_term::Colour;

use logos::Logos;

use text_io::try_read;

fn get_ln(s: &String) -> Option<usize> {
    match {
        let res: Result<usize, _> = try_read!(s.bytes());
        res
    } {
        Ok(ret) => Some(ret),
        Err(..) => None,
    }
}

struct State<'a> {
    buffer: &'a mut BTreeMap<usize, String>,
}

struct Helper {}

impl rustyline::Helper for Helper {}

#[derive(Logos, Debug, PartialEq)]
enum Token<'a> {
    #[regex(r"\d+")]
    Number(&'a str),

    #[regex(r#""[^"]*""#)]
    String(&'a str),

    #[regex("(LET)|(PRINT)|(END)|(FOR)|(NEXT)|(GOTO)|(GOSUB)|(RETURN)|(IF)|(THEN)|(DEF)|(READ)|(DATA)|(DIM)|REM")]
    Keyword(&'a str),

    #[regex(r"[+\-*/^]")]
    Operator(&'a str),

    #[regex("[a-zA-Z]+")]
    Ident(&'a str),

    #[regex(r"\n")]
    Newline,

    #[regex(" ")]
    Space,

    #[regex(r"\t")]
    Tab,

    #[error]
    Error,
}

impl Highlighter for Helper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        std::borrow::Cow::Owned(highlight(line))
    }
}

impl Completer for Helper {
    type Candidate = String;

    fn complete(
        &self,
        _line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // TODO: Completion
        Ok((0, vec![]))
    }
}

impl Hinter for Helper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }
        let start = if ctx.history_index() == ctx.history().len() {
            ctx.history_index().saturating_sub(1)
        } else {
            ctx.history_index()
        };
        if let Some(sr) = ctx
            .history()
            .starts_with(line, start, SearchDirection::Reverse)
        {
            if sr.entry == line {
                return None;
            }
            return Some(sr.entry[pos..].to_owned());
        }
        None
    }
}

impl Validator for Helper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(validate_brackets(ctx.input()))
    }
}

fn validate_brackets(input: &str) -> ValidationResult {
    let mut stack = vec![];
    for c in input.chars() {
        match c {
            '(' | '[' | '{' => stack.push(c),
            ')' | ']' | '}' => match (stack.pop(), c) {
                (Some('('), ')') | (Some('['), ']') | (Some('{'), '}') => {}
                (Some(wanted), _) => {
                    return ValidationResult::Invalid(Some(format!(
                        "Mismatched brackets: {:?} is not properly closed",
                        wanted
                    )))
                }
                (None, c) => {
                    return ValidationResult::Invalid(Some(format!(
                        "Mismatched brackets: {:?} is unpaired",
                        c
                    )))
                }
            },
            _ => {}
        }
    }
    if stack.is_empty() {
        ValidationResult::Valid(None)
    } else {
        ValidationResult::Incomplete
    }
}

fn highlight(buf: &str) -> String {
    let mut res: Vec<String> = vec![];

    let lex = Token::lexer(buf);

    for tok in lex {
        match tok {
            Token::Number(num) => res.push(Colour::Red.paint(num).to_string()),
            Token::String(str) => res.push(Colour::Purple.paint(str).to_string()),
            Token::Keyword(kw) => res.push(Colour::Yellow.paint(kw).to_string()),
            Token::Operator(op) => res.push(Colour::Blue.paint(op.to_string()).to_string()),
            Token::Ident(id) => res.push(Colour::Cyan.paint(id).to_string()),
            Token::Newline => res.push("\n".to_string()),
            Token::Space => res.push(" ".to_string()),
            Token::Tab => res.push("\t".to_string()),
            _ => (),
        }
    }

    res.join("")
}

fn process_input(state: &mut State, input: String) {
    if let Some(ln) = get_ln::<usize>(&input, "{}") {
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
        } else if let Some(w) = get_ln::<String>(&input, "w {}") {
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
            println!(
                "d{}",
                highlight(
                    state
                        .buffer
                        .values()
                        .cloned()
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_str()
                )
            );
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
    let mut rl = rustyline::Editor::<Helper>::new()?;
    rl.set_color_mode(rustyline::ColorMode::Enabled);
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
