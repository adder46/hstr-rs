use ncurses::{attr_t, NCURSES_ATTR_T, WINDOW};

#[allow(non_snake_case)]
pub const fn A_BOLD() -> attr_t {
    0
}

#[allow(non_snake_case)]
pub fn LINES() -> i32 {
    10
}

#[allow(non_snake_case)]
pub fn COLS() -> i32 {
    80
}

#[allow(non_snake_case)]
pub fn COLOR_PAIR(_n: i16) -> attr_t {
    0
}

pub const COLOR_BLACK: i16 = 0;
pub const COLOR_RED: i16 = 1;
pub const COLOR_GREEN: i16 = 2;
pub const COLOR_CYAN: i16 = 6;
pub const COLOR_WHITE: i16 = 7;

pub fn attron(_a: NCURSES_ATTR_T) -> i32 {
    0
}

pub fn attroff(_a: NCURSES_ATTR_T) -> i32 {
    0
}

pub fn clear() -> i32 {
    0
}

pub fn mvaddstr(_y: i32, _x: i32, _s: &str) -> i32 {
    0
}

pub fn mvaddch(_y: i32, _x: i32, _c: chtype) -> i32 {
    0
}

pub fn start_color() -> i32 {
    0
}

pub fn init_pair(_pair: i16, _f: i16, _b: i16) -> i32 {
    0
}

pub fn setlocale(_lc: LcCategory, _locale: &str) -> String {
    String::new()
}

pub fn initscr() -> WINDOW {
    std::ptr::null_mut::<i8>()
}

pub fn noecho() -> i32 {
    0
}

pub fn keypad(_w: WINDOW, _bf: bool) -> i32 {
    0
}

pub fn stdscr() -> WINDOW {
    std::ptr::null_mut::<i8>()
}

#[allow(non_camel_case_types)]
pub type chtype = u64;

pub fn refresh() -> i32 {
    0
}

pub fn doupdate() -> i32 {
    0
}

pub fn endwin() -> i32 {
    0
}

pub fn getch() -> i32 {
    0
}

pub fn get_wch() -> Option<WchResult> {
    Some(WchResult::Char(0))
}

#[allow(non_camel_case_types)]
pub enum LcCategory {
    all,
}

pub enum WchResult {
    Char(u32),
    KeyCode(i32),
}

pub const KEY_DOWN: i32 = 0x102;
pub const KEY_UP: i32 = 0x103;
pub const KEY_BACKSPACE: i32 = 0x107;
pub const KEY_DC: i32 = 0x14a;
pub const KEY_NPAGE: i32 = 0x152;
pub const KEY_PPAGE: i32 = 0x153;
pub const KEY_RESIZE: i32 = 0x19a;
