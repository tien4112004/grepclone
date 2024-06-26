use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use super::utils::{parse_context, Colors, ContextType};
use crate::core::error::CliError;
use crate::core::flag::Flags;
use crate::core::print_with_context::{print_after_context, print_before_context, print_context};
use crate::core::utils::build_regex;
use crate::getwriter;

fn count_matches<T: BufRead + Sized>(reader: T, re: Regex) -> u32 {
    let mut matches: u32 = 0;
    reader.lines().for_each(|line| {
        re.find(&line.unwrap()).map(|_| matches += 1);
    });

    matches
}

fn print_matches<T: BufRead + Sized>(
    reader: T,
    re: Regex,
    flags: &Flags,
    mut writer: impl Write,
) -> Result<(), CliError> {
    let mut lines = reader.lines().enumerate();
    while let Some((i, Ok(line))) = lines.next() {
        if re.find(&line).is_none() {
            continue;
        };

        let match_iter = re.find_iter(&line);
        let mut matched_line = line.clone();

        match (flags.highlight, flags.line_number) {
            (true, true) => {
                for mat in match_iter {
                    matched_line = format!(
                        "{}: {}",
                        Colors::colorize_text(Colors::Green, &format!("{}", i + 1)),
                        re.replace_all(
                            &matched_line,
                            Colors::colorize_text(Colors::Red, mat.as_str())
                        )
                    );
                }
            }

            (true, false) => {
                for mat in match_iter {
                    matched_line = format!(
                        "{}",
                        re.replace_all(
                            &matched_line,
                            Colors::colorize_text(Colors::Red, mat.as_str())
                        )
                    );
                }
            }

            (false, true) => {
                matched_line = format!(
                    "{}: {}",
                    Colors::colorize_text(Colors::Green, &format!("{}", i + 1)),
                    re.replace_all(&matched_line, Colors::colorize_text(Colors::Red, "$0"))
                );
            }

            _ => (), // the rest
        }
        writeln!(writer, "{}", matched_line)?;
    }
    writer.flush()?;
    Ok(())
}

fn print_invert_matches<T: BufRead + Sized>(
    reader: T,
    re: Regex,
    flags: &Flags,
    mut writer: impl Write,
) -> Result<(), CliError> {
    let mut lines = reader.lines().enumerate();
    while let Some((i, Ok(line))) = lines.next() {
        if re.find(&line).is_some() {
            continue;
        };

        if flags.line_number {
            writeln!(
                writer,
                "{}: {}",
                Colors::colorize_text(Colors::Green, &format!("{}", i + 1)),
                line
            )?;
        } else {
            writeln!(writer, "{}", line)?;
        }
    }

    writer.flush()?;
    Ok(())
}

// TODO: The function goes to stdin all the time. Fix this.
pub(crate) fn prepare_and_choose(
    needle: (&str, bool),
    path: &std::path::Path,
    flags: &Flags,
    context: ContextType,
    group_seperator: &str,
) -> Result<(), CliError> {
    let re = build_regex(needle.0, needle.1)?;
    if path == Path::new("STDIN") {
        let stdin = io::stdin();
        let stdin_reader = BufReader::new(stdin.lock());
        let writer = getwriter!();
        choose_process(stdin_reader, re, writer, flags, context, group_seperator)?;
    } else {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let writer = getwriter!();
        choose_process(reader, re, writer, flags, context, group_seperator)?;
    }

    Ok(())
}

fn choose_process<T: BufRead + Sized>(
    reader: T,
    re: regex::Regex,
    writer: impl Write,
    flags: &Flags,
    context: ContextType,
    group_separator: &str,
) -> Result<(), CliError> {
    if flags.count {
        println!("{}", count_matches(reader, re));
        return Ok(());
    } else if flags.invert_match {
        print_invert_matches(reader, re, flags, writer)?;
        return Ok(());
    }

    match context {
        ContextType::Before(before_ctx) => {
            let before_ctx = parse_context(before_ctx)?;
            print_before_context(reader, re, flags, before_ctx, group_separator, writer)?
        }

        ContextType::After(after_ctx) => {
            let after_ctx = parse_context(after_ctx)?;
            print_after_context(reader, re, flags, after_ctx, group_separator, writer)?
        }

        ContextType::Context(ctx) => {
            let ctx = parse_context(ctx)?;
            print_context(reader, re, flags, ctx, group_separator, writer)?;
        }

        ContextType::None => {
            print_matches(reader, re, flags, writer)?;
        }
    }

    Ok(())
}
