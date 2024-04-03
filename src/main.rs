use std::path::Path;

mod core;

use crate::core::cli::Cli;
use crate::core::{flag::Flags, process::prepare_and_choose};

fn main() {
    let args = Cli::new().parse();

    let pattern = args.value_of("pattern").unwrap();
    let input = Path::new(args.value_of("input").unwrap_or("STDIN"));
    let group_separator = args.value_of("group-separator").unwrap_or("---");

    println!("Debug: Pattern: {}", pattern);
    println!("Debug: Input: {:?}", input);
    println!("Debug: Group Separator: {}", group_separator);

    let flags = Flags::set_flags(&args);

    if let Err(e) = prepare_and_choose((pattern, flags.ignore_case), input, &flags, group_separator)
    {
        fatal!("Error: {e}");
    }
}
