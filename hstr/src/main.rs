#[cfg(test)]
use fake_ncurses as nc;
#[cfg(not(test))]
use ncurses as nc;

mod app;
mod cli;
mod sort;
mod ui;
mod util;

#[allow(unreachable_code)]
fn main() -> Result<(), std::io::Error> {
    if let Some(arg) = cli::parse_args() {
        util::print_config(&arg);
        return Ok(());
    }
    ui::curses::init();
    ui::curses::init_color_pairs();
    let mut user_interface = ui::UserInterface::new();
    user_interface.populate_screen();
    loop {
        let user_input = nc::get_wch();
        user_interface.handle_input(user_input)?;
    }
    ui::curses::teardown();
    Ok(())
}
