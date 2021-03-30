use crate::state::View;
use crate::ui::Direction;

#[cfg(test)]
use fake_ncurses as nc;
#[cfg(not(test))]
use ncurses as nc;

use structopt::StructOpt;
use unicode_width::UnicodeWidthChar;

mod hstr;
mod io;
mod sort;
mod state;
mod ui;

const CTRL_E: u32 = 5;
const CTRL_F: u32 = 6;
const TAB: u32 = 9;
const ENTER: u32 = 10;
const CTRL_T: u32 = 20;
const ESC: u32 = 27;
const CTRL_SLASH: u32 = 31;
const Y: i32 = b'y' as i32;

#[derive(Debug, StructOpt)]
struct Opt {
    query: Vec<String>,
    #[structopt(name = "show-config", long)]
    show_config: Option<String>,
}

fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();
    if let Some(shell) = opt.show_config {
        io::print_config(&shell);
        return Ok(());
    }

    let query = opt.query.join(" ");
    let mut state = state::State::new(&query);
    let mut user_interface = ui::UserInterface::new(&query);

    ui::curses::init();
    state.search();
    state
        .query
        .clone()
        .chars()
        .for_each(|_| user_interface.move_cursor(&mut state, Direction::Forward));
    user_interface.populate_screen(&state);

    loop {
        let user_input = nc::get_wch();
        match user_input.unwrap() {
            nc::WchResult::Char(ch) => match ch {
                CTRL_E => {
                    state.toggle_search_mode();
                    user_interface.selected = 0;
                    user_interface.populate_screen(&state);
                }
                CTRL_F => match user_interface.selected(&state) {
                    Some(command) => {
                        if state.view == View::Favorites {
                            user_interface.retain_selected(&state);
                        }
                        state.add_or_rm_fav(command);
                        io::write_to_home(
                            &format!(".config/hstr-rs/.{}_favorites", state.shell),
                            state.commands(View::Favorites),
                        )?;
                        nc::clear();
                        user_interface.populate_screen(&state);
                    }
                    None => continue,
                },
                TAB => match user_interface.selected(&state) {
                    Some(command) => {
                        io::echo(command);
                        break;
                    }
                    None => continue,
                },
                ENTER => match user_interface.selected(&state) {
                    Some(command) => {
                        io::echo(command + "\n");
                        break;
                    }
                    None => continue,
                },
                CTRL_T => {
                    state.toggle_case();
                    user_interface.populate_screen(&state);
                }
                ESC => break,
                CTRL_SLASH => {
                    state.toggle_view();
                    user_interface.selected = 0;
                    user_interface.page = 1;
                    nc::clear();
                    user_interface.populate_screen(&state);
                }
                _ => {
                    let query_length_in_bytes = state
                        .query
                        .chars()
                        .take(user_interface.cursor.chars_moved)
                        .fold(0, |acc, x| acc + x.to_string().len());
                    state
                        .query
                        .insert(query_length_in_bytes, std::char::from_u32(ch).unwrap());
                    state.commands = state.to_restore.clone();
                    user_interface.cursor.query_char_widths = state
                        .query
                        .chars()
                        .map(|ch| ch.width().unwrap_or(0))
                        .collect::<Vec<usize>>();
                    user_interface.selected = 0;
                    user_interface.page = 1;
                    nc::clear();
                    state.search();
                    user_interface.populate_screen(&state);
                    user_interface.move_cursor(&mut state, Direction::Forward);
                }
            },
            nc::WchResult::KeyCode(code) => match code {
                nc::KEY_LEFT => {
                    user_interface.move_cursor(&mut state, Direction::Backward);
                }
                nc::KEY_RIGHT => {
                    user_interface.move_cursor(&mut state, Direction::Forward);
                }
                nc::KEY_UP => {
                    user_interface.move_selected(&state, Direction::Backward);
                    user_interface.populate_screen(&state);
                }
                nc::KEY_DOWN => {
                    user_interface.move_selected(&state, Direction::Forward);
                    user_interface.populate_screen(&state);
                }
                nc::KEY_BACKSPACE => {
                    let mut new_query_string = String::new();
                    ui::column_indices(&state.query.clone()).for_each(|(colidx, _byteidx, _ch)| {
                        if user_interface.cursor.column != colidx + _ch.width().unwrap_or(0) {
                            new_query_string.push(_ch);
                        }
                    });
                    state.query = new_query_string;
                    state.commands = state.to_restore.clone();
                    user_interface.cursor.query_char_widths = ui::get_char_widths(&state.query);
                    nc::clear();
                    state.search();
                    user_interface.populate_screen(&state);
                    user_interface.move_cursor(&mut state, Direction::Backward);
                }
                nc::KEY_DC => match user_interface.selected(&state) {
                    Some(command) => {
                        user_interface.ask_before_deletion(&command);
                        if nc::getch() == Y {
                            user_interface.retain_selected(&state);
                            state.delete_from_history(command);
                            io::write_to_home(
                                &format!(".{}_history", state.shell),
                                &state.raw_history,
                            )?;
                        }
                        state.reload_history();
                        nc::clear();
                        user_interface.populate_screen(&state);
                    }
                    None => continue,
                },
                nc::KEY_NPAGE => {
                    user_interface.turn_page(&state, Direction::Forward);
                    user_interface.populate_screen(&state);
                }
                nc::KEY_PPAGE => {
                    user_interface.turn_page(&state, Direction::Backward);
                    user_interface.populate_screen(&state);
                }
                nc::KEY_RESIZE => {
                    nc::clear();
                    user_interface.populate_screen(&state);
                }
                _ => {}
            },
        }
    }

    ui::curses::teardown();

    Ok(())
}
