use libc::{ioctl, TIOCSTI};
use regex::Regex;
use std::env;
use std::fs::{create_dir_all, write, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<String>, std::io::Error> {
    let p = dirs::home_dir().unwrap().join(path);
    if p.exists() {
        let file = File::open(p)?;
        let reader = BufReader::new(file);
        reader.lines().collect::<Result<Vec<_>, _>>()
    } else {
        Ok(Vec::new())
    }
}

pub fn write_file(path: impl AsRef<Path>, thing: &[String]) -> Result<(), std::io::Error> {
    let p = dirs::home_dir().unwrap().join(path);
    if !p.exists() {
        create_dir_all(p.parent().unwrap())?;
        File::create(&p)?;
    }
    write(p, thing.join("\n"))?;
    Ok(())
}

pub fn echo(command: String) {
    for byte in command.as_bytes() {
        unsafe {
            ioctl(0, TIOCSTI, byte);
        }
    }
}

pub fn get_shell_prompt() -> String {
    format!(
        "{}@{}$",
        env::var("USER").unwrap(),
        gethostname::gethostname().into_string().unwrap()
    )
}

pub fn substring_indices<'a>(string: &'a str, substring: &'a str) -> Vec<usize> {
    match Regex::new(substring) {
        Ok(r) => r.find_iter(string).flat_map(|m| m.range()).collect(),
        Err(_) => vec![],
    }
}

pub fn print_config(shell: &str) {
    match shell {
        "bash" => {
            let bash_config = include_str!("config/bash");
            println!("{}", bash_config);
        }
        "zsh" => {
            let zsh_config = include_str!("config/zsh");
            println!("{}", zsh_config);
        }
        "N/A" => println!("Available options: bash, zsh"),
        _ => {}
    }
}

pub mod zsh_history {
    use regex::Regex;
    use std::fs::File;
    use std::io::{self, Read};

    pub fn process() -> String {
        let history = read().unwrap();
        let unmetafied = unmetafy(history);
        remove_timestamps(String::from_utf8(unmetafied).unwrap())
    }

    fn unmetafy(mut bytestring: Vec<u8>) -> Vec<u8> {
        /* Unmetafying zsh history requires looping over the bytestring, removing
         * each encountered Meta character, and XOR-ing the following byte with 32.
         *
         * For instance:
         *
         * Input: ('a', 'b', 'c', Meta, 'd', 'e', 'f')
         * Wanted: ('a', 'b', 'c', 'd' ^ 32, 'e', 'f')
         */
        const ZSH_META: u8 = 0x83;
        for index in (0..bytestring.len()).rev() {
            if bytestring[index] == ZSH_META {
                bytestring.remove(index);
                bytestring[index] ^= 32;
            }
        }
        bytestring
    }

    fn read() -> Result<Vec<u8>, io::Error> {
        let path = dirs::home_dir().unwrap().join(".zsh_history");
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn remove_timestamps(history: String) -> String {
        /* The preceding metadata needs to be stripped
         * because zsh history entries look like below:
         *
         * `: 1330648651:0;sudo reboot`
         */
        let r = Regex::new(r"^: \d{10}:\d;").unwrap();
        history.lines().map(|x| r.replace(x, "") + "\n").collect()
    }
}
