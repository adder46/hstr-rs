use std::env;
use std::fs::{create_dir_all, File, write};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use libc::{ioctl, TIOCSTI};

pub fn read_file(path: &str) -> Result<Vec<String>, std::io::Error> {
    let p = dirs::home_dir().unwrap().join(PathBuf::from(path));
    if !Path::new(p.as_path()).exists() {
        create_dir_all(p.parent().unwrap());
        File::create(p.as_path());
        Ok(Vec::new())
    } else {
        let file = File::open(p).unwrap();
        let reader = BufReader::new(file);
        let contents = reader.lines().collect::<Result<Vec<_>, _>>();
        contents
    }
}

pub fn write_file(path: &str, thing: &Vec<String>) {
    let p = dirs::home_dir().unwrap().join(PathBuf::from(path));
    write(p, thing.join("\n"));
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
