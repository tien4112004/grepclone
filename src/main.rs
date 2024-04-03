use core::utils::ContextType;
use std::path::Path;

mod core;

use crate::core::cli::Cli;
use crate::core::{flag::Flags, process::prepare_and_choose};

fn main() {
    let args = Cli::new().parse();

    let pattern = args.value_of("pattern").unwrap();
    let input = Path::new(args.value_of("input").unwrap_or("STDIN"));
    let group_separator = args.value_of("group-separator").unwrap_or("---");

    let flags = Flags::set_flags(&args);

    let contextType = match (
        args.value_of("before-context"),
        args.value_of("after-context"),
        args.value_of("both-context"),
    ) {
        (Some(before), None, None) => ContextType::Before(before),
        (None, Some(after), None) => ContextType::After(after),
        (None, None, Some(both)) => ContextType::Both(both),
        _ => ContextType::None,
    };

    if let Err(e) = prepare_and_choose(
        (pattern, flags.ignore_case),
        input,
        &flags,
        contextType,
        group_separator,
    ) {
        fatal!("Error: {e}");
    }
}
