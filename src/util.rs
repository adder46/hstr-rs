use std::env;
use std::fs::{create_dir_all, File, write};
use std::io::{BufRead, BufReader};
use std::ops::{Add, Rem};
use std::path::{Path, PathBuf};
use libc::{c_char, gethostname, ioctl, sysconf, TIOCSTI, _SC_HOST_NAME_MAX};

pub fn read_file(path: &str) -> Vec<String> {
    let p = dirs::home_dir().unwrap().join(PathBuf::from(path));
    if !Path::new(p.as_path()).exists() {
        create_dir_all(p.parent().unwrap());
        File::create(p.as_path());
        Vec::new()
    } else {
        let file = File::open(p).unwrap();
        let reader = BufReader::new(file);
        let contents = reader.lines().map(|l| l.unwrap()).collect();
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
    let hostname_max = unsafe { sysconf(_SC_HOST_NAME_MAX) };
    let mut buffer = vec![0 as u8; (hostname_max as usize) + 1];
    unsafe {
        gethostname(buffer.as_mut_ptr() as *mut c_char, buffer.len());
    }
    let end = buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or_else(|| buffer.len());
    buffer.resize(end, 0);
    format!("{}@{}$ ", env::var("USER").unwrap(), String::from_utf8(buffer).unwrap())
}

pub fn modulo<T>(a: T, b: T) -> T
where
    T: Copy + Add<T, Output=T> + Rem<T, Output=T>,
{
    ((a % b) + b) % b
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_modulo() {
        let a = -21;
        let b = 4;
        assert_eq!(super::modulo(a, b), 3);
    }
}