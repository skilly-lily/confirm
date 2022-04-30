use std::convert::Infallible;
use std::num::NonZeroU8;
use std::io::{stdin, stdout, Write};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Answer {
    Yes,
    No,
    Retry,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ReaderType {
    SingleChar,
    NewlineBuffered,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TryMode {
    Infinite,
    Count(NonZeroU8),
}

impl FromStr for Answer {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let low = s.to_ascii_lowercase();
        match low.as_str() {
            "yes" | "y" => Ok(Answer::Yes),
            "no" | "n" => Ok(Answer::No),
            _ => Ok(Answer::Retry),
        }
    }
}

fn is_full_word(s: &str) -> bool {
    ["yes", "no"].contains(&s.to_ascii_lowercase().as_str())
}

fn parse_default_answer_opt(s: &str) -> Result<Answer> {
    if !is_full_word(s) && s != "retry" {
        Err(anyhow!(format!(
            "Invalid choice, choose either yes or no, found: {}",
            s
        )))
    } else {
        Ok(Answer::from_str(s)?)
    }
}

fn parse_retry_count_opt(s: &str) -> Result<TryMode> {
    use TryMode::*;

    let count: u8 = s.parse()?;
    if let Some(nz) = NonZeroU8::new(count) {
        Ok(Count(nz))
    } else {
        Ok(Infinite)
    }
}

/// Get user confirmation
#[derive(Clone, Debug, Parser)]
#[clap(name = "confirm")]
struct MainOptions {
    /// Require explicit "yes" or "no", not single letters.
    ///
    /// Cannot be used with --no-enter.
    #[clap(short, long)]
    full_words: bool,

    /// Choose a default answer
    ///
    /// If no default is chosen, and the user supplies an empty answer, then a
    /// retry is triggered.  Otherwise, the default is used on an empty answer.
    /// If the retry count has been hit, then the process assumes a negative
    /// response and exits 1. Using the keyword "retry" is identical to
    /// omitting the option.
    #[clap(short, long, default_value = "retry", parse(try_from_str = parse_default_answer_opt))]
    default: Answer,

    /// Don't require newlines
    ///
    /// Read the character on the terminal as it's typed, without waiting for
    /// the user to hit enter/return.  
    #[clap(long, conflicts_with = "full_words")]
    no_enter: bool,

    /// Number of times to ask
    ///
    /// Number of total times a question should be asked.  Use 0 for infinite
    /// retries.
    #[clap(short, long, default_value = "3", parse(try_from_str = parse_retry_count_opt))]
    ask_count: TryMode,

    /// The prompt to display
    ///
    /// Prompt of "Continue?" will become "Continue? [y/n]: ".  Options are
    /// added and highlighted based on given settings.  Original message will
    /// NEVER be modified.
    #[clap(name = "PROMPT", default_value = "Continue?")]
    prompt: String,

    /// Don't ask any question, return successfully.
    /// 
    /// Turns the entire tool into a no-op, useful when building shell scripts
    /// around the tool.
    #[clap(long = "--yes")]
    always_yes: bool,

    /// Don't ask any question, fail immediately.
    /// 
    /// Turns the tool into no-op failure.  Useful when testing shell scripts
    /// built around this tool.
    #[clap(long = "--no")]
    always_no: bool,
}

impl MainOptions {
    fn into_confirm(self) -> Confirm {
        let reader_type = match self.no_enter {
            true => ReaderType::SingleChar,
            false => ReaderType::NewlineBuffered,
        };
        Confirm::new(
            self.default,
            self.prompt,
            reader_type,
            self.ask_count,
            self.full_words,
        )
    }
}

#[derive(Debug, Clone)]
struct Confirm {
    default_response: Answer,
    prompt: String,
    reader_type: ReaderType,
    retry_mode: TryMode,
    use_full_words: bool,
}

impl Confirm {
    pub fn new(
        default_response: Answer,
        prompt: String,
        reader_type: ReaderType,
        retry_mode: TryMode,
        use_full_words: bool,
    ) -> Self {
        Self {
            default_response,
            reader_type,
            prompt,
            use_full_words,
            retry_mode,
        }
    }

    fn render_option_box(&self) -> &'static str {
        use Answer::*;
        match (self.use_full_words, self.default_response) {
            (true, Yes) => "[YES/no]",
            (true, No) => "[yes/NO]",
            (true, Retry) => "[yes/no]",
            (false, Yes) => "[Y/n]",
            (false, No) => "[y/N]",
            (false, Retry) => "[y/n]",
        }
    }

    fn prepare_prompt(&self) -> String {
        let optionbox = self.render_option_box();
        let mut new = self.prompt.clone();
        new.push(' ');
        new.push_str(optionbox);
        new.push_str(": ");
        new
    }

    fn try_read_value(&self, prompt: &str) -> Result<Answer> {
        use ReaderType::*;
        print!("{}", prompt);
        stdout().flush()?;
        let mut input_buf = String::new();
        match self.reader_type {
            NewlineBuffered => {
                stdin().read_line(&mut input_buf)?;
            }
            SingleChar => {
                let ch = getch::Getch::new().getch()?;
                println!();
                input_buf.push(ch as char);
            }
        };

        let response = input_buf.trim();

        if response.is_empty() {
            Ok(self.default_response)
        } else if self.use_full_words && !is_full_word(response) {
            Err(anyhow!("Please type yes or no"))
        } else {
            Ok(Answer::from_str(response)?)
        }
    }

    fn get_user_input(&self, prompt: &str) -> Answer {
        self.try_read_value(prompt).unwrap_or_else(|err| {
            eprintln!("Error while reading user input: {}", err);
            Answer::Retry
        })
    }

    pub fn ask_loop(&self) -> bool {
        let prompt = self.prepare_prompt();

        macro_rules! ask {
            () => {
                match self.get_user_input(&prompt) {
                    Answer::Yes => {
                        return true;
                    }
                    Answer::No => {
                        return false;
                    }
                    Answer::Retry => {}
                };
            };
        }

        ask!(); // We always ask it at least once.

        match self.retry_mode {
            TryMode::Infinite => loop {
                ask!();
            },
            TryMode::Count(x) => {
                for _ in 0..x.get() {
                    ask!();
                }
                eprintln!("Retry count exceeded.  Aborting...");
                false
            }
        }
    }
}

impl From<MainOptions> for Confirm {
    fn from(o: MainOptions) -> Self {
        o.into_confirm()
    }
}

fn main() {
    if atty::isnt(atty::Stream::Stdin) {
        eprintln!("Warning: using confirm when stdin is not a tty is not supported.");
    }
    let opts = MainOptions::parse();
    if opts.always_yes {
        return;
    } else if opts.always_no {
        std::process::exit(1)
    }
    let confirmed = Confirm::from(opts).ask_loop();
    if !confirmed {
        std::process::exit(1);
    };
}
