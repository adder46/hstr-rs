use crate::state::View;
use crate::ui::Direction;

#[cfg(test)]
use fake_ncurses as nc;
#[cfg(not(test))]
use ncurses as nc;

use std::cell::RefCell;
use std::rc::Rc;
use structopt::StructOpt;

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

#[allow(unreachable_code)]
fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();
    if let Some(shell) = opt.show_config {
        io::print_config(&shell);
        return Ok(());
    }

    let query = opt.query.join(" ");
    let state = Rc::new(RefCell::new(state::State::new(query)));
    let mut user_interface = ui::UserInterface::new(Rc::clone(&state));

    ui::curses::init();
    state.borrow_mut().search();
    user_interface.populate_screen();

    loop {
        let user_input = nc::get_wch();
        match user_input.unwrap() {
            nc::WchResult::Char(ch) => match ch {
                CTRL_E => {
                    state.borrow_mut().toggle_search_mode();
                    user_interface.selected = 0;
                    user_interface.populate_screen();
                }
                CTRL_F => match user_interface.selected() {
                    Some(command) => {
                        if state.borrow().view == View::Favorites {
                            user_interface.retain_selected();
                        }
                        state.borrow_mut().add_or_rm_fav(command);
                        io::write_to_home(
                            &format!(".config/hstr-rs/.{}_favorites", state.borrow().shell),
                            state.borrow().commands(View::Favorites),
                        )?;
                        nc::clear();
                        user_interface.populate_screen();
                    }
                    None => continue,
                },
                TAB => match user_interface.selected() {
                    Some(command) => {
                        io::echo(command);
                        break;
                    }
                    None => continue,
                },
                ENTER => match user_interface.selected() {
                    Some(command) => {
                        io::echo(command + "\n");
                        break;
                    }
                    None => continue,
                },
                CTRL_T => {
                    state.borrow_mut().toggle_case();
                    user_interface.populate_screen();
                }
                ESC => break,
                CTRL_SLASH => {
                    state.borrow_mut().toggle_view();
                    user_interface.selected = 0;
                    user_interface.page = 1;
                    nc::clear();
                    user_interface.populate_screen();
                }
                _ => {
                    state
                        .borrow_mut()
                        .query
                        .push(std::char::from_u32(ch).unwrap());
                    user_interface.selected = 0;
                    user_interface.page = 1;
                    nc::clear();
                    state.borrow_mut().search();
                    user_interface.populate_screen();
                }
            },
            nc::WchResult::KeyCode(code) => match code {
                nc::KEY_UP => {
                    user_interface.move_selected(Direction::Backward);
                    user_interface.populate_screen();
                }
                nc::KEY_DOWN => {
                    user_interface.move_selected(Direction::Forward);
                    user_interface.populate_screen();
                }
                nc::KEY_BACKSPACE => {
                    let mut st = state.borrow_mut();
                    st.query.pop();
                    st.commands = st.to_restore.clone();
                    nc::clear();
                    st.search();
                    drop(st);
                    user_interface.populate_screen();
                }
                nc::KEY_DC => match user_interface.selected() {
                    Some(command) => {
                        user_interface.ask_before_deletion(&command);
                        if nc::getch() == Y {
                            user_interface.retain_selected();
                            state.borrow_mut().delete_from_history(command);
                            io::write_to_home(
                                &format!(".{}_history", state.borrow().shell),
                                &state.borrow().raw_history,
                            )?;
                        }
                        state.borrow_mut().reload_history();
                        nc::clear();
                        user_interface.populate_screen();
                    }
                    None => continue,
                },
                nc::KEY_NPAGE => {
                    user_interface.turn_page(Direction::Forward);
                    user_interface.populate_screen();
                }
                nc::KEY_PPAGE => {
                    user_interface.turn_page(Direction::Backward);
                    user_interface.populate_screen();
                }
                nc::KEY_RESIZE => {
                    nc::clear();
                    user_interface.populate_screen();
                }
                _ => {}
            },
        }
    }

    ui::curses::teardown();

    Ok(())
}
