use libc::{fileno, ioctl, FILE, TIOCSTI};
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

pub fn echo(f: *mut FILE, command: String) {
    unsafe {
        for byte in command.as_bytes() {
            ioctl(fileno(f), TIOCSTI, byte);
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
        .map(|x| r.replace(x, "").into())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn zsh_unmetafy(mut contents: Vec<u8>) -> Vec<u8> {
    for index in (0..contents.len()).rev() {
        if contents[index] == 0x83 {
            contents.remove(index);
            contents[index] ^= 32;
        }
    }
    contents.to_vec()
}
