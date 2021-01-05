use libc::{fileno, ioctl, FILE, TIOCSTI};
use std::env;
use std::fs::{create_dir_all, write, File};
use std::io::{BufRead, BufReader};
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
