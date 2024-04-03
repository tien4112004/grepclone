use crate::core::error::CliError;
use owo_colors::OwoColorize;
use regex::RegexBuilder;

#[macro_export]
macro_rules! getwriter {
    () => {{
        let stdout = std::io::stdout();
        let handle = stdout.lock();
        let writer = std::io::BufWriter::new(handle);
        writer
    }};
}

pub(crate) enum Colors {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Colors {
    pub(crate) fn colorize_pattern(color: Self, pattern: &str) -> String {
        match color {
            Colors::Red => pattern.red().to_string(),
            Colors::Green => pattern.green().to_string(),
            Colors::Blue => pattern.blue().to_string(),
            Colors::Yellow => pattern.yellow().to_string(),
        }
    }
}

pub(crate) fn build_regex(pattern: &str, ignore_case: bool) -> Result<regex::Regex, CliError> {
    let re = match ignore_case {
        true => RegexBuilder::new(pattern).case_insensitive(true).build()?,

        false => RegexBuilder::new(pattern).build()?,
    };

    Ok(re)
}

#[derive(Clone, Copy)]
pub(crate) enum ContextType<'context> {
    After(&'context str),
    Before(&'context str),
    Context(&'context str),
    None,
}

pub(crate) fn parse_context(context: &str) -> Result<usize, CliError> {
    context.parse::<usize>().map_err(|e| e.into())
}
