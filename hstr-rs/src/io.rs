use libc::{ioctl, TIOCSTI};
use std::{
    fs::{create_dir_all, write, File},
    io::{BufRead, BufReader, Error, Read},
    path::{Path, PathBuf},
};

pub fn read_as_bytes() -> Result<Vec<u8>, Error> {
    let home = dirs::home_dir().unwrap();
    let path = home.join(".zsh_history");
    let mut file = File::open(path)?;
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

fn ensure_target_existence(target: &PathBuf) -> Result<(), Error> {
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
