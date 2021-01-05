use std::ffi::CStr;
use std::{ffi::CString, path::Path, ptr};

use libc::c_void;

use history_error::HistoryError;

use crate::ext_readline::{add_history, history_list};

mod history_error;

mod ext_readline {
    use std::ffi::CStr;
    use std::fmt;
    use std::fmt::{Display, Formatter};

    use libc::{c_char, c_int, c_void};

    #[repr(C)]
    #[derive(Debug)]
    pub struct HistoryEntry {
        pub line: *const c_char,
        pub data: *mut c_void,
    }

    impl Display for HistoryEntry {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let line = unsafe { CStr::from_ptr(self.line) };
            let line = line.to_str().unwrap();
            write!(f, "{}", line)
        }
    }

    #[link(name = "readline")]
    extern "C" {
        pub fn add_history(entry: *const c_char) -> *mut c_void;
        pub fn remove_history(which: c_int) -> *mut HistoryEntry;
        pub fn free_history_entry(entry: *mut HistoryEntry) -> *mut c_void;
        pub fn write_history(file: *const c_char) -> c_int;
        pub fn history_list() -> *const *const HistoryEntry;
    }
}

pub fn display_history_list() {
    println!("History:");
    unsafe {
        let list = history_list();
        let mut i = 0;
        loop {
            let entry = *list.offset(i);
            if entry.is_null() {
                break;
            }

            let line = CStr::from_ptr((*entry).line);
            println!("  {}", line.to_str().unwrap());
            i += 1;
        }
    }
    println!();
}

pub fn add(entry: &str) {
    let entry = CString::new(entry).unwrap();
    unsafe {
        add_history(entry.as_ptr());
    }
}

pub fn remove<'a>(offset: i32) -> &'a mut ext_readline::HistoryEntry {
    unsafe { &mut *ext_readline::remove_history(offset) }
}

pub fn free_entry(entry: &mut ext_readline::HistoryEntry) -> Result<(), *mut c_void> {
    unsafe {
        let data_ptr = ext_readline::free_history_entry(entry);

        if data_ptr.is_null() {
            Ok(())
        } else {
            Err(data_ptr)
        }
    }
}

pub fn write(path: Option<&Path>) -> Result<i32, HistoryError> {
    with_path_ptr(path, |ptr| unsafe {
        history_error::gen_result(ext_readline::write_history(ptr))
    })
}

fn with_path_ptr<F>(path: Option<&Path>, f: F) -> Result<i32, HistoryError>
where
    F: Fn(*const i8) -> Result<i32, HistoryError>,
{
    if let Some(p) = path {
        match p.to_str() {
            Some(p) => {
                if let Ok(cs) = CString::new(p) {
                    return f(cs.as_ptr());
                }
            }
            None => {
                return Err(HistoryError::new(
                    "History Error",
                    "Unable to determine path!",
                ))
            }
        }
    }
    f(ptr::null())
}
