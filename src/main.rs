use ncurses::*;

mod app;
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

fn main() {
    initscr();
    noecho();
    keypad(stdscr(), true);
    let mut app = app::Application::new(
        0,
        false,
        false,
        String::new()
    );
    let mut user_interface = ui::UserInterface::new();
    user_interface.init_color_pairs();
    user_interface.populate_screen(&app);
    loop {
        let user_input = get_wch();
        match user_input.unwrap() {
            WchResult::Char(ch) => {
                match ch {
                    CTRL_E => {
                        app.toggle_match();
                        user_interface.set_selected(0);
                        user_interface.populate_screen(&app);
                    },
                    CTRL_F => {
                        let entries = app.get_entries(app.view());
                        let command = user_interface.get_selected(&entries);
                        app.add_to_or_remove_from_favorites(command);
                    },
                    TAB => {
                        let entries = app.get_entries(app.view());
                        let command = user_interface.get_selected(&entries);
                        util::echo(command);
                        break;
                    },
                    ENTER => {
                    let entries = app.get_entries(app.view());
                    let command = user_interface.get_selected(&entries);
                        util::echo(command);
                        util::echo("\n".to_string());
                        break;
                    },
                    CTRL_T => {
                        app.toggle_case();
                        user_interface.populate_screen(&app);
                    },
                    ESC => break, // ESC
                    CTRL_SLASH => {
                        app.toggle_view();
                        user_interface.set_selected(0);
                        user_interface.populate_screen(&app);
                    }
                    _ => {
                        app.search_string_mut().push(std::char::from_u32(ch as u32).unwrap());
                        user_interface.set_selected(0);
                        user_interface.set_page(1);               
                        app.search();
                        user_interface.populate_screen(&app);
                    }
                }
            },
            WchResult::KeyCode(code) => {
                match code {
                    KEY_UP => {
                        let entries = app.get_entries(app.view());
                        user_interface.move_selected(entries, -1);
                        user_interface.populate_screen(&app);
                    },
                    KEY_DOWN => {
                        let entries = app.get_entries(app.view());
                        user_interface.move_selected(entries, 1);
                        user_interface.populate_screen(&app);
                    },
                    KEY_BACKSPACE => {
                        app.search_string_mut().pop();
                        app.set_all_entries(app.to_restore().clone());
                        app.search();
                        user_interface.populate_screen(&app);
                    },
                    KEY_DC => {
                        let entries = app.get_entries(app.view());
                        let command = user_interface.get_selected(&entries);
                        user_interface.prompt_for_deletion(&command);
                        app.delete_from_history(command);
                        app = app::Application::new(
                            app.view(), 
                            app.regex_match(),
                            app.case_sensitivity(),
                            String::from(app.search_string())
                        );
                        user_interface.populate_screen(&app);
                    }
                    _ => {}
                }
            }
        }
    }
    endwin();
}
