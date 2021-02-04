use libc::{ioctl, TIOCSTI};
use regex::Regex;
use std::env;
use std::fs::{create_dir_all, write, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

pub fn read_file(path: String) -> Result<Vec<String>, std::io::Error> {
    let p = dirs::home_dir().unwrap().join(PathBuf::from(path));
    if !Path::new(p.as_path()).exists() {
        create_dir_all(p.parent().unwrap())?;
        File::create(p.as_path())?;
        Ok(Vec::new())
    } else {
        let file = File::open(p).unwrap();
        let reader = BufReader::new(file);
        reader.lines().collect::<Result<Vec<_>, _>>()
    }
}

pub fn write_file(path: String, thing: &[String]) -> Result<(), std::io::Error> {
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

pub fn read_zsh_history() -> Result<String, io::Error> {
    let mut file = File::open(dirs::home_dir().unwrap().join(".zsh_history"))?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let unmetafied_history = zsh_unmetafy(contents);
    Ok(remove_timestamps(
        String::from_utf8(unmetafied_history).unwrap(),
    ))
}

fn remove_timestamps(contents: String) -> String {
    let r = Regex::new(r"^: \d{10}:\d;").unwrap();
    contents
        .lines()
        .map(|x| format!("{}\n", r.replace(x, "")))
        .collect()
}

pub fn zsh_unmetafy(mut contents: Vec<u8>) -> Vec<u8> {
    (0..contents.len()).rev().for_each(|index| {
        if contents[index] == 0x83 {
            contents.remove(index);
            contents[index] ^= 32;
        }
    });
    contents
}

pub fn print_config_bash() {
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

pub fn print_config_zsh() {
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
