use crate::app::{Application, View};
use crate::ui::{Direction, UserInterface};
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
const Y: i32 = b'Y' as i32;

fn main() -> Result<(), std::io::Error> {
    if let Some(arg) = cli::parse_args() {
        util::print_config(arg);
        return Ok(());
    }
    ui::curses::init();
    let shell = get_shell().get_name();
    let mut application = Application::new(shell);
    application.load_history();
    ui::curses::init_color_pairs();
    let mut user_interface = UserInterface::new();
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
                    let selected = user_interface.selected;
                    let command = user_interface.page.selected(&commands, selected);
                    if application.view == View::Favorites {
                        user_interface.retain_selected(&commands);
                    }
                    application.add_or_rm_fav(command);
                    util::write_file(
                        &format!(".config/hstr-rs/.{}_favorites", shell),
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
                    let selected = user_interface.selected;
                    let command = user_interface.page.selected(&commands, selected);
                    util::echo(command);
                    break;
                }
                ENTER => {
                    let commands = application.get_commands();
                    let selected = user_interface.selected;
                    let command = user_interface.page.selected(&commands, selected);
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
                    user_interface.page.value = 1;
                    nc::clear();
                    user_interface.populate_screen(&application);
                }
                _ => {
                    application
                        .search_string
                        .push(std::char::from_u32(ch).unwrap());
                    user_interface.selected = 0;
                    user_interface.page.value = 1;
                    nc::clear();
                    application.search();
                    user_interface.populate_screen(&application);
                }
            },
            nc::WchResult::KeyCode(code) => match code {
                nc::KEY_UP => {
                    let commands = application.get_commands();
                    user_interface.move_selected(commands, Direction::Backward);
                    user_interface.populate_screen(&application);
                }
                nc::KEY_DOWN => {
                    let commands = application.get_commands();
                    user_interface.move_selected(commands, Direction::Forward);
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
                    let selected = user_interface.selected;
                    let command = user_interface.page.selected(&commands, selected);
                    user_interface.ask_before_deletion(&command);
                    if nc::getch() == Y {
                        user_interface.retain_selected(&commands);
                        application.delete_from_history(command);
                        util::write_file(&format!(".{}_history", shell), &application.raw_history)?;
                    }
                    application.reload_history();
                    nc::clear();
                    user_interface.populate_screen(&application);
                }
                nc::KEY_NPAGE => {
                    let commands = application.get_commands();
                    user_interface.turn_page(commands, Direction::Forward);
                    user_interface.populate_screen(&application);
                }
                nc::KEY_PPAGE => {
                    let commands = application.get_commands();
                    user_interface.turn_page(commands, Direction::Backward);
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
    ui::curses::teardown();
    Ok(())
}
