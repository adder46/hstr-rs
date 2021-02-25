mod app;
mod cli;
mod hstr;
mod io;
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
    let mut user_interface = ui::UserInterface::new();
    user_interface.populate_screen();
    user_interface.mainloop()?;
    ui::curses::teardown();
    Ok(())
}
