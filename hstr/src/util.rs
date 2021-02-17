use libc::{ioctl, TIOCSTI};
use regex::Regex;
use std::env;
use std::fs::{create_dir_all, write, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

pub fn read_file(path: &str) -> Result<Vec<String>, std::io::Error> {
    let p = dirs::home_dir().unwrap().join(PathBuf::from(path));
    if Path::new(p.as_path()).exists() {
        let file = File::open(p).unwrap();
        let reader = BufReader::new(file);
        reader.lines().collect::<Result<Vec<_>, _>>()
    } else {
        create_dir_all(p.parent().unwrap())?;
        File::create(p.as_path())?;
        Ok(Vec::new())
    }
}

pub fn write_file(path: &str, thing: &[String]) -> Result<(), std::io::Error> {
    let p = dirs::home_dir().unwrap().join(PathBuf::from(path));
    write(p, thing.join("\n"))?;
    Ok(())
}

pub fn echo(command: String) {
    unsafe {
        for byte in command.as_bytes() {
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

pub fn zsh_process_history() -> String {
    let history = zsh_read_history().unwrap();
    let unmetafied = zsh_unmetafy_history(history);
    zsh_remove_timestamps(String::from_utf8(unmetafied).unwrap())
}

fn zsh_unmetafy_history(mut bytestring: Vec<u8>) -> Vec<u8> {
    /* Unmetafying zsh history requires looping over the bytestring, removing
     * each encountered Meta character, and XOR-ing the following byte with 32.
     *
     * For instance:
     *
     * Input: ('a', 'b', 'c', Meta, 'd', 'e', 'f')
     * Wanted: ('a', 'b', 'c', 'd' ^ 32, 'e', 'f')
     */
    const ZSH_META: u8 = 0x83;
    (0..bytestring.len()).rev().for_each(|index| {
        if bytestring[index] == ZSH_META {
            bytestring.remove(index);
            bytestring[index] ^= 32;
        }
    });
    bytestring
}

fn zsh_read_history() -> Result<Vec<u8>, io::Error> {
    let path = dirs::home_dir().unwrap().join(".zsh_history");
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn zsh_remove_timestamps(history: String) -> String {
    /* The preceding metadata needs to be stripped
     * because zsh history entries look like below:
     *
     * `: 1330648651:0;sudo reboot`
     */
    let r = Regex::new(r"^: \d{10}:\d;").unwrap();
    history.lines().map(|x| r.replace(x, "") + "\n").collect()
}

pub fn print_config(sh: String) {
    match sh.as_str() {
        "bash" => print_config_bash(),
        "zsh" => print_config_zsh(),
        _ => {}
    }
}

fn print_config_bash() {
    println!(
        "\
        # append new history items to .bash_history\n\
        shopt -s histappend\n\
        # don't put duplicate lines or lines starting with space in the history\n\
        HISTCONTROL=ignoreboth\n\
        # increase history file size\n\
        HISTFILESIZE=1000000\n\
        # increase history size\n\
        HISTSIZE=${{HISTFILESIZE}}\n\
        # sync entries in memory with .bash_history\n\
        export PROMPT_COMMAND=\"history -a; history -n; ${{PROMPT_COMMAND}}\"\n\
        # bind hstr-rs to CTRL + H\n\
        if [[ $- =~ .*i.* ]]; then bind '\"\\C-h\": \"hstr-rs \\C-j\"'; fi"
    );
}

fn print_config_zsh() {
    println!(
        "\
        # append new history items to .bash_history\n\
        setopt INC_APPEND_HISTORY\n\
        # don't put duplicate lines\n\
        setopt HIST_IGNORE_ALL_DUPS\n\
        # don't put lines starting with space in the history\n\
        setopt HIST_IGNORE_SPACE\n\
        # increase history file size\n\
        HISTFILESIZE=1000000\n\
        # increase history size\n\
        HISTSIZE=${{HISTFILESIZE}}\n\
        # bind hstr-rs to CTRL + H\n\
        bindkey -s '^H' 'hstr-rs^M'"
    );
}
