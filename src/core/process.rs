use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use super::utils::{parse_context, Colors, ContextType};
use crate::core::error::CliError;
use crate::core::flag::Flags;
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
                        Colors::colorize_pattern(Colors::Green, &format!("{}", i + 1)),
                        re.replace_all(
                            &matched_line,
                            Colors::colorize_pattern(Colors::Red, mat.as_str())
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
                            Colors::colorize_pattern(Colors::Red, mat.as_str())
                        )
                    );
                }
            }

            (false, true) => {
                matched_line = format!(
                    "{}: {}",
                    Colors::colorize_pattern(Colors::Green, &format!("{}", i + 1)),
                    re.replace_all(&matched_line, Colors::colorize_pattern(Colors::Red, "$0"))
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
                Colors::colorize_pattern(Colors::Green, &format!("{}", i + 1)),
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

        ContextType::Both(both_ctx) => {
            let both_ctx = parse_context(both_ctx)?;
            print_before_context(reader, re, flags, both_ctx, group_separator, writer)?;
            // print_after_context(reader, re, flags, both_ctx, group_separator, writer)?;
        }

        ContextType::None => {
            print_matches(reader, re, flags, writer)?;
        }
    }

    Ok(())
}

fn print_before_context<T: BufRead + Sized>(
    reader: T,
    re: Regex,
    flags: &Flags,
    context_size: usize,
    group_separator: &str,
    mut writer: impl Write,
) -> Result<(), CliError> {
    let lines = reader.lines().collect::<std::io::Result<Vec<String>>>()?;

    let mut matched_line_numbers: Vec<usize> = Vec::with_capacity(lines.len());
    let mut matched_lines: Vec<Vec<(usize, String)>> = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        if re.find(line).is_none() {
            continue;
        }

        matched_line_numbers.push(i);
        let v = Vec::with_capacity(context_size + 1);
        matched_lines.push(v);
    }

    for (j, matched_number) in matched_line_numbers.iter().enumerate() {
        for (i, line) in lines.iter().enumerate() {
            let starting_point = matched_number.saturating_sub(context_size);
            if i >= starting_point && i <= *matched_number {
                let mut matched_line = line.clone();
                let match_iter = re.find_iter(line);

                match_iter.for_each(|matched| {
                    matched_line = re
                        .replace_all(
                            &matched_line,
                            Colors::colorize_pattern(Colors::Red, matched.as_str()),
                        )
                        .to_string()
                });

                matched_lines[j].push((i, matched_line));
            } else {
                matched_lines[j].push((i, line.clone()));
            }
        }
    }

    for (matched_line, is_last, is_first) in matched_lines
        .iter()
        .enumerate()
        .map(|(index, m)| (m, index == matched_lines.len(), index == 0))
    {
        if !is_first && !is_last {
            writeln!(
                writer,
                "{}",
                Colors::colorize_pattern(Colors::Yellow, group_separator)
            )?;
        }

        if flags.line_number {
            for (i, line) in matched_line.iter() {
                writeln!(
                    writer,
                    "{}: {}",
                    Colors::colorize_pattern(Colors::Green, &format!("{}", i + 1)),
                    line
                )?;
            }
        } else {
            for (_, line) in matched_line.iter() {
                writeln!(writer, "{}", line)?;
            }
        }
    }

    writer.flush()?;
    Ok(())
}

fn print_after_context<T: BufRead + Sized>(
    reader: T,
    re: Regex,
    flags: &Flags,
    context_size: usize,
    group_separator: &str,
    mut writer: impl Write,
) -> Result<(), CliError> {
    let lines = reader.lines().collect::<std::io::Result<Vec<String>>>()?;

    let mut matched_line_numbers: Vec<usize> = Vec::with_capacity(lines.len());
    let mut matched_lines: Vec<Vec<(usize, String)>> = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        if re.find(line).is_none() {
            continue;
        }

        matched_line_numbers.push(i);
        let v = Vec::with_capacity(context_size + 1);
        matched_lines.push(v);
    }

    for (j, matched_number) in matched_line_numbers.iter().enumerate() {
        for (i, line) in lines.iter().enumerate() {
            let ending_point = matched_number + context_size;
            if (i >= *matched_number) && (i <= ending_point) {
                if (i == *matched_number) && (flags.highlight) {
                    let mut matched_line = line.clone();
                    re.find_iter(line).for_each(|matched| {
                        matched_line = re
                            .replace_all(
                                &matched_line,
                                Colors::colorize_pattern(Colors::Red, matched.as_str()),
                            )
                            .to_string();
                    });
                    matched_lines[j].push((i, matched_line));
                }
            } else {
                matched_lines[j].push((i, line.clone()));
            }
        }
    }

    for (matched_line, is_last, is_first) in matched_lines
        .iter()
        .enumerate()
        .map(|(index, m)| (m, index == matched_lines.len(), index == 0))
    {
        if !is_first && !is_last {
            writeln!(
                writer,
                "{}",
                Colors::colorize_pattern(Colors::Yellow, group_separator)
            )?;
        }

        if flags.line_number {
            for (i, line) in matched_line.iter() {
                writeln!(
                    writer,
                    "{}: {}",
                    Colors::colorize_pattern(Colors::Green, &format!("{}", i + 1)),
                    line
                )?;
            }
        } else {
            for (_, line) in matched_line.iter() {
                writeln!(writer, "{}", line)?;
            }
        }
    }

    writer.flush()?;
    Ok(())
}
