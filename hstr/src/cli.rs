use clap::{App, Arg};

pub fn parse_args() -> Option<String> {
    let matches = App::new("hstr-rs")
        .version("0.7.0")
        .author("adder46 <dedmauz69@gmail.com>")
        .about("History suggest box for bash and zsh")
        .arg(
            Arg::with_name("show-config")
                .long("show-config")
                .takes_value(true)
                .value_name("SHELL"),
        )
        .get_matches();
    match matches.value_of("show-config") {
        Some(a) => match a {
            "bash" | "zsh" => Some(a.to_string()),
            _ => Some("N/A".to_string()),
        },
        None => None,
    }
}
