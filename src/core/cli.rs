use clap::{App, Arg, ArgMatches};

pub(crate) struct Cli<'cli> {
    app: App<'cli>,
}

impl<'cli> Cli<'cli> {
    pub(crate) fn new() -> Self {
        let app = App::new("grepclone")
            .version("0.1")
            .author("phanttien")
            .about("Searches for patterns. Prints lines that match those patterns to the standard output.")
            .arg(
                Arg::with_name("pattern")
                    .help("The pattern to search for")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("input")
                    .help("[OPTIONAL] File to search in. If omitted, takes input from STDIN.")
                    .takes_value(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("count")
                    .help("print only a count of selected lines per FILE")
                    .short('c')
                    .long("count")
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::with_name("line-number")
                    .help("print line number with output line")
                    .short('n')
                    .long("line-number")
                    .takes_value(false)
                    .required(false)
                )
            .arg(
                Arg::with_name("highlight")
                    .help("Highlight matched words.")
                    .short('l')
                    .long("highlight")
                    .takes_value(false)
                    .required(false)
            )
            .arg(
                Arg::with_name("ignore-case")
                    .help("Ignore case distinction.")
                    .short('i')
                    .long("ignore-case")
                    .takes_value(false)
                    .required(false)
            )
            .arg(
                Arg::with_name("invert-match")
                    .help("Select non-matching lines.")
                    .short('v')
                    .long("invert-match")
                    .takes_value(false)
                    .required(false)
            )
            .arg(
                Arg::with_name("group-separator")
                    .help("Use SEP as the group separator. By default, SEP is a triple hyphen (---).")
                    .short('s')
                    .long("group-separator")
                    .value_name("SEP")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                Arg::with_name("before-context")
                .help("print NUM lines of leading context")
                .short('B')
                .long("before-context")
                .value_name("NUM")
                .takes_value(true)
                .required(false)
            )
            .arg(
                Arg::with_name("after-context")
                .help("print NUM lines of trailing context")
                .short('A')
                .long("after-context")
                .value_name("NUM")
                .takes_value(true)
                .required(false)
            )
            .arg(
                Arg::with_name("context")
                .help("print NUM lines of output context")
                .short('C')
                .long("context")
                .value_name("NUM")
                .takes_value(true)
                .required(false)
            );

        Self { app }
    }

    pub(crate) fn parse(self) -> ArgMatches {
        self.app.get_matches()
    }
}
