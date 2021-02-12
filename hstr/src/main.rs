use crate::app::{Application, View};
use crate::ui::UserInterface;
use ncurses as nc;
use setenv::get_shell;

mod app;
mod cli;
mod sort;
mod ui;
mod util;

const CTRL_E: u32 = 5;
const CTRL_F: u32 = 6;
const TAB: u32 = 9;
const ENTER: u32 = 10;
const CTRL_T: u32 = 20;
const ESC: u32 = 27;
const CTRL_SLASH: u32 = 31;
const Y: i32 = 121;

fn main() -> Result<(), std::io::Error> {
    if let Some(arg) = cli::parse_args() {
        match arg.as_str() {
            "bash" | "zsh" => {
                util::print_config(arg);
            }
            _ => {}
        }
        return Ok(());
    }
    nc::setlocale(nc::LcCategory::all, "");
    nc::initscr();
    nc::noecho();
    nc::keypad(nc::stdscr(), true);
    let shell = get_shell().get_name();
    let mut application = Application::new(shell);
    application.load_commands()?;
    let mut user_interface = UserInterface::new();
    user_interface.init_color_pairs();
    user_interface.populate_screen(&application);
    loop {
        let user_input = nc::get_wch();
        match user_input.unwrap() {
            nc::WchResult::Char(ch) => match ch {
                CTRL_E => {
                    application.toggle_regex_mode();
                    user_interface.selected = 0;
                    user_interface.populate_screen(&application);
                }
                CTRL_F => {
                    let commands = application.get_commands();
                    let command = user_interface.get_selected(&commands);
                    if application.view == View::Favorites {
                        user_interface.retain_selection(&commands);
                    }
                    application.add_or_rm_fav(command);
                    util::write_file(
                        format!(".config/hstr-rs/.{}_favorites", shell),
                        application
                            .commands
                            .as_ref()
                            .unwrap()
                            .get(&View::Favorites)
                            .unwrap(),
                    )?;
                    nc::clear();
                    user_interface.populate_screen(&application);
                }
                TAB => {
                    let commands = application.get_commands();
                    let command = user_interface.get_selected(&commands);
                    util::echo(command);
                    break;
                }
                ENTER => {
                    let commands = application.get_commands();
                    let command = user_interface.get_selected(&commands);
                    util::echo(format!("{}\n", command));
                    break;
                }
                CTRL_T => {
                    application.toggle_case();
                    user_interface.populate_screen(&application);
                }
                ESC => break,
                CTRL_SLASH => {
                    application.toggle_view();
                    user_interface.selected = 0;
                    user_interface.page = 1;
                    nc::clear();
                    user_interface.populate_screen(&application);
                }
                _ => {
                    application
                        .search_string
                        .push(std::char::from_u32(ch).unwrap());
                    user_interface.selected = 0;
                    user_interface.page = 1;
                    nc::clear();
                    application.search();
                    user_interface.populate_screen(&application);
                }
            },
            nc::WchResult::KeyCode(code) => match code {
                nc::KEY_UP => {
                    let commands = application.get_commands();
                    user_interface.move_selected(commands, -1);
                    user_interface.populate_screen(&application);
                }
                nc::KEY_DOWN => {
                    let commands = application.get_commands();
                    user_interface.move_selected(commands, 1);
                    user_interface.populate_screen(&application);
                }
                nc::KEY_BACKSPACE => {
                    application.search_string.pop();
                    application.restore();
                    nc::clear();
                    application.search();
                    user_interface.populate_screen(&application);
                }
                nc::KEY_DC => {
                    let commands = application.get_commands();
                    let command = user_interface.get_selected(&commands);
                    user_interface.prompt_for_deletion(&command);
                    if nc::getch() == Y {
                        user_interface.retain_selection(&commands);
                        application.delete_from_history(command);
                        util::write_file(format!(".{}_history", shell), &application.raw_history)?;
                    }
                    application.reload_commands();
                    nc::clear();
                    user_interface.populate_screen(&application);
                }
                nc::KEY_NPAGE => {
                    let commands = application.get_commands();
                    user_interface.turn_page(commands, 1);
                    user_interface.populate_screen(&application);
                }
                nc::KEY_PPAGE => {
                    let commands = application.get_commands();
                    user_interface.turn_page(commands, -1);
                    user_interface.populate_screen(&application);
                }
                nc::KEY_RESIZE => {
                    nc::clear();
                    user_interface.populate_screen(&application);
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
