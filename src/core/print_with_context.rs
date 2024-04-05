use super::utils::Colors;
use crate::core::error::CliError;
use crate::core::flag::Flags;
use regex::Regex;
use std::io::{BufRead, Write};

fn print_line_number(
    line_number: usize,
    // matched: bool,
    writer: &mut impl Write,
) -> Result<(), CliError> {
    // let separate_char = if matched { ":" } else { "-" };
    write!(
        writer,
        "{}: ",
        Colors::colorize_text(Colors::Green, &format!("{}", line_number)),
        // separate_char = true
    )?;
    Ok(())
}

fn colorize_matched(i: usize, line: &str, re: &Regex) -> (usize, String) {
    let mut matched_line = line.to_string();
    let match_iter = re.find_iter(line);
    // let highlight_color = color.unwrap_or(Colors::Red);

    match_iter.for_each(|matched| {
        matched_line = re
            .replace_all(
                &matched_line,
                Colors::colorize_text(Colors::Red, matched.as_str()),
            )
            .to_string()
    });

    (i, matched_line)
}

// fn print_matched_lines(
//     matched_lines: Vec<Vec<(usize, String)>>,
//     group_separator: &str,
//     flags: &Flags,
//     writer: &mut impl Write,
// ) {
//     for (matched_line, is_last, is_first) in matched_lines
//         .iter()
//         .enumerate()
//         .map(|(index, m)| (m, index == matched_lines.len(), index == 0))
//     {
//         if !is_first && !is_last {
//             writeln!(
//                 writer,
//                 "{}",
//                 Colors::colorize_text(Colors::Yellow, group_separator)
//             )?;
//         }

//         for (i, line) in matched_line.iter() {
//             if flags.line_number {
//                 print_line_number(i + 1, writer);
//             }
//             writeln!(writer, "{}", line)?;
//         }
//     }
// }

pub fn print_before_context<T: BufRead + Sized>(
    reader: T,
    re: Regex,
    flags: &Flags,
    context_size: usize,
    group_separator: &str,
    mut writer: impl Write,
) -> Result<(), CliError> {
    let lines = reader.lines().collect::<std::io::Result<Vec<String>>>()?;

    let mut matched_lines: Vec<Vec<(usize, String)>> = Vec::new();

    let mut current_group: Vec<(usize, String)> = Vec::new();
    let mut last_line_added = 0;

    for (i, line) in lines.iter().enumerate() {
        if re.find(line).is_none() {
            continue;
        }

        if let Some((last_index, _)) = current_group.last() {
            if *last_index + context_size < i {
                matched_lines.push(current_group.clone());
                current_group = Vec::with_capacity(context_size * 2 + 1);
            }
        }

        let start = i.saturating_sub(context_size);
        for j in start.max(last_line_added)..i {
            let (line_number, context_line) = colorize_matched(j, &lines[j], &re);
            current_group.push((line_number, context_line));
        }

        let (i, matched_line) = colorize_matched(i, &line, &re);
        current_group.push((i, matched_line));
        last_line_added = i + 1;
    }

    if !current_group.is_empty() {
        matched_lines.push(current_group);
    }

    for (index, matched_line) in matched_lines.iter().enumerate() {
        let is_first = index == 0;
        let is_last = index == matched_lines.len() - 1;

        if !is_first {
            writeln!(
                writer,
                "{}",
                Colors::colorize_text(Colors::Yellow, group_separator)
            )?;
        }

        for (i, line) in matched_line.iter() {
            if flags.line_number {
                print_line_number(i + 1, &mut writer)?;
            }
            writeln!(writer, "{}", line)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn print_after_context<T: BufRead + Sized>(
    reader: T,
    re: Regex,
    flags: &Flags,
    context_size: usize,
    group_separator: &str,
    mut writer: impl Write,
) -> Result<(), CliError> {
    let lines = reader.lines().collect::<std::io::Result<Vec<String>>>()?;

    let mut matched_lines: Vec<Vec<(usize, String)>> = Vec::with_capacity(lines.len());
    let mut current_group: Vec<(usize, String)> = Vec::new();
    let mut last_line_added: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        if re.find(line).is_none() {
            continue;
        }

        if let Some((last_index, _)) = current_group.last() {
            if *last_index + context_size < i {
                matched_lines.push(current_group.clone());
                current_group = Vec::with_capacity(context_size * 2 + 1);
            }
        }

        if i >= last_line_added {
            let (i, matched_line) = colorize_matched(i, &line, &re);
            current_group.push((i, matched_line));
            last_line_added = i;
        }

        let end = (i + context_size).min(lines.len() - 1);
        for j in last_line_added..=end {
            if j <= i {
                continue;
            }
            let (line_number, context_line) = colorize_matched(j, &lines[j], &re);
            current_group.push((line_number, context_line));
        }

        last_line_added = i + context_size + 1;
    }

    if !current_group.is_empty() {
        matched_lines.push(current_group);
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
                Colors::colorize_text(Colors::Yellow, group_separator)
            )?;
        }

        for (i, line) in matched_line.iter() {
            if flags.line_number {
                print_line_number(i + 1, &mut writer)?;
            }
            writeln!(writer, "{}", line)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn print_context<T: BufRead + Sized>(
    reader: T,
    re: Regex,
    flags: &Flags,
    context_size: usize,
    group_separator: &str,
    mut writer: impl Write,
) -> Result<(), CliError> {
    let lines = reader
        .lines()
        .collect::<std::io::Result<Vec<String>>>()
        .unwrap();

    let mut matched_line_numbers: Vec<usize> = Vec::with_capacity(lines.len());
    let mut matched_lines: Vec<Vec<(usize, String)>> = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        if re.find(line).is_none() {
            continue;
        }

        matched_line_numbers.push(i);
        let v = Vec::with_capacity(context_size + 2);
        matched_lines.push(v);
    }

    for (j, matched_number) in matched_line_numbers.iter().enumerate() {
        for (i, line) in lines.iter().enumerate() {
            let starting_point = matched_number.saturating_sub(context_size);
            let ending_point = matched_number + context_size;

            if i < starting_point || i > ending_point {
                continue;
            }

            if flags.highlight {
                let (i, matched_line) = colorize_matched(i, &line, &re);
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
                Colors::colorize_text(Colors::Yellow, group_separator)
            )
            .unwrap();
        }

        for (i, line) in matched_line.iter() {
            if flags.line_number {
                print_line_number(i + 1, &mut writer)?;
            }
            writeln!(writer, "{}", line)?;
        }
    }

    writer.flush()?;
    Ok(())
}
