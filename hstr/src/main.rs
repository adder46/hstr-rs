use crate::app::{Application, View};
use crate::ui::UserInterface;
use crate::util::write_file;
use ncurses as nc;
use setenv::get_shell;
use std::ffi::CString;

mod app;
mod sort;
mod ui;
mod util;

const CTRL_C: u32 = 3;
const CTRL_E: u32 = 5;
const CTRL_F: u32 = 6;
const TAB: u32 = 9;
const ENTER: u32 = 10;
const CTRL_T: u32 = 20;
const ESC: u32 = 27;
const CTRL_SLASH: u32 = 31;
const Y: i32 = 121;

fn main() -> Result<(), std::io::Error> {
    nc::setlocale(nc::LcCategory::all, "");
    let device = CString::new("/dev/tty").unwrap();
    let device_c = device.as_ptr() as *const libc::c_char;
    let mode = CString::new("r+").unwrap();
    let mode_c = mode.as_ptr() as *const libc::c_char;
    let f = unsafe { libc::fopen(device_c, mode_c) };
    nc::newterm(None, f, f);
    nc::noecho();
    nc::raw();
    nc::keypad(nc::stdscr(), true);
    let shell = get_shell().get_name();
    let mut app = Application::new(shell);
    app.load_commands()?;
    let mut user_interface = UserInterface::new();
    user_interface.init_color_pairs();
    user_interface.populate_screen(&app);
    loop {
        let user_input = nc::get_wch();
        match user_input.unwrap() {
            nc::WchResult::Char(ch) => match ch {
                CTRL_C => {
                    if app.dirty_history {
                        match shell {
                            "bash" => {
                                util::echo(f, "history -r".to_string());
                                util::echo(f, "\n".to_string());
                            }
                            "zsh" => {
                                util::echo(f, "fc -R".to_string());
                                util::echo(f, "\n".to_string());
                            }
                            _ => {}
                        }
                    }
                    break;
                }
                CTRL_E => {
                    app.toggle_regex_mode();
                    user_interface.selected = 0;
                    user_interface.populate_screen(&app);
                }
                CTRL_F => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    if app.view == View::Favorites {
                        let page_size = user_interface.get_page_size(&commands) - 1;
                        if user_interface.selected == page_size {
                            user_interface.selected -= 1;
                        }
                    }
                    app.add_or_rm_fav(command);
                    write_file(
                        format!(".config/hstr-rs/.{}_favorites", shell),
                        app.commands
                            .as_ref()
                            .unwrap()
                            .get(&app::View::Favorites)
                            .unwrap(),
                    )?;
                    nc::clear();
                    user_interface.populate_screen(&app);
                }
                TAB => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    util::echo(f, command);
                    break;
                }
                ENTER => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    util::echo(f, command);
                    util::echo(f, "\n".to_string());
                    break;
                }
                CTRL_T => {
                    app.toggle_case();
                    user_interface.populate_screen(&app);
                }
                ESC => {
                    if app.dirty_history {
                        match shell {
                            "bash" => {
                                util::echo(f, "history -r".to_string());
                                util::echo(f, "\n".to_string());
                            }
                            "zsh" => {
                                util::echo(f, "fc -R".to_string());
                                util::echo(f, "\n".to_string());
                            }
                            _ => {}
                        }
                    }
                    break;
                }
                CTRL_SLASH => {
                    app.toggle_view();
                    user_interface.selected = 0;
                    nc::clear();
                    user_interface.populate_screen(&app);
                }
                _ => {
                    app.search_string.push(std::char::from_u32(ch).unwrap());
                    user_interface.selected = 0;
                    user_interface.page = 1;
                    nc::clear();
                    app.search();
                    user_interface.populate_screen(&app);
                }
            },
            nc::WchResult::KeyCode(code) => match code {
                nc::KEY_UP => {
                    let commands = app.get_commands();
                    user_interface.move_selected(commands, -1);
                    user_interface.populate_screen(&app);
                }
                nc::KEY_DOWN => {
                    let commands = app.get_commands();
                    user_interface.move_selected(commands, 1);
                    user_interface.populate_screen(&app);
                }
                nc::KEY_BACKSPACE => {
                    app.search_string.pop();
                    app.restore();
                    nc::clear();
                    app.search();
                    user_interface.populate_screen(&app);
                }
                nc::KEY_DC => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    user_interface.prompt_for_deletion(&command);
                    if nc::getch() == Y {
                        app.delete_from_history(command)?;
                    }
                    app.reload_commands();
                    nc::clear();
                    user_interface.populate_screen(&app);
                }
                nc::KEY_NPAGE => {
                    let commands = app.get_commands();
                    user_interface.turn_page(commands, 1);
                    user_interface.populate_screen(&app);
                }
                nc::KEY_PPAGE => {
                    let commands = app.get_commands();
                    user_interface.turn_page(commands, -1);
                    user_interface.populate_screen(&app);
                }
                _ => {}
            },
        }
    }
    nc::clear();
    nc::refresh();
    nc::doupdate();
    nc::endwin();
    Ok(())
}
