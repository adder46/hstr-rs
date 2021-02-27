use crate::io::print_config;
use clap::{App, Arg};

pub fn parse_args() -> String {
    let matches = App::new("hstr-rs")
        .version("0.8.0")
        .author("adder46 <dedmauz69@gmail.com>")
        .about("History suggest box for bash and zsh")
        .arg(
            Arg::with_name("show-config")
                .long("show-config")
                .takes_value(true)
                .value_name("shell"),
        )
        .arg(
            Arg::with_name("shell")
                .possible_values(&["bash", "zsh"])
                .required_unless("show-config"),
        )
        .get_matches();

    if let Some(arg) = matches.value_of("show-config") {
        print_config(arg);
        std::process::exit(0);
    }

    matches.value_of("shell").unwrap().to_string()
}
