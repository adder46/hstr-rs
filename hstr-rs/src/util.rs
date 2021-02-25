use regex::Regex;

pub fn print_config(shell: &str) {
    match shell {
        "bash" => print_bash_config(),
        "zsh" => print_zsh_config(),
        _ => println!("Available options: bash, zsh"),
    }
}

fn print_bash_config() {
    let bash_config = include_str!("config/bash");
    println!("{}", bash_config);
}

fn print_zsh_config() {
    let zsh_config = include_str!("config/zsh");
    println!("{}", zsh_config);
}

pub fn substring_indices<'a>(string: &'a str, substring: &'a str) -> Vec<usize> {
    match Regex::new(substring) {
        Ok(r) => r.find_iter(string).flat_map(|m| m.range()).collect(),
        Err(_) => vec![],
    }
}
