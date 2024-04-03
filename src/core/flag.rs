use clap::ArgMatches;

#[derive(Debug, Default)]
pub struct Flags {
    pub count: bool,
    pub line_number: bool,
    pub highlight: bool,
    pub ignore_case: bool,
    pub no_match: bool,
}

impl Flags {
    pub fn new() -> Self {
        Flags::default()
    }

    pub fn set_flags(a: &ArgMatches) -> Self {
        let mut flags = Flags::new();

        flags.count = a.is_present("count");
        flags.line_number = a.is_present("line-number");
        flags.highlight = a.is_present("highlight");
        flags.ignore_case = a.is_present("ignore-case");
        flags.no_match = a.is_present("no-match");

        flags
    }
}
