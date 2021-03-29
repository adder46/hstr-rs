use libc::{ioctl, TIOCSTI};
use std::{
    fs::{create_dir_all, write, File},
    io::{BufRead, BufReader, Error, Read},
    path::{Path, PathBuf},
};

pub fn read_as_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
    let home = dirs::home_dir().unwrap();
    let target = home.join(path);
    let mut file = File::open(target)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn read_from_home(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
    /* `path` is relative to home directory */
    let home = dirs::home_dir().unwrap();
    let target = home.join(path);
    if target.exists() {
        read_file(target)
    } else {
        Ok(Vec::new())
    }
}

fn read_file(target: PathBuf) -> Result<Vec<String>, Error> {
    let file = File::open(target)?;
    let reader = BufReader::new(file);
    reader.lines().collect::<Result<Vec<_>, _>>()
}

pub fn write_to_home(path: impl AsRef<Path>, thing: &[String]) -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();
    let target = home.join(path);
    ensure_target_existence(&target)?;
    write(target, thing.join("\n"))?;
    Ok(())
}

fn ensure_target_existence(target: &Path) -> Result<(), Error> {
    if !target.exists() {
        create_dir_all(target.parent().unwrap())?;
        File::create(target)?;
    }
    Ok(())
}

pub fn echo(command: String) {
    for byte in command.as_bytes() {
        unsafe {
            ioctl(0, TIOCSTI, byte);
        }
    }
}

pub fn print_config(shell: &str) {
    match shell {
        "bash" => print_bash_config(),
        "zsh" => print_zsh_config(),
        _ => eprintln!("Available options: bash, zsh"),
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
