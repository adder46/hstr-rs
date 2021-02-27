mod app;
mod cli;
mod hstr;
mod io;
mod sort;
mod ui;

#[allow(unreachable_code)]
fn main() -> Result<(), std::io::Error> {
    let sh = cli::parse_args();
    ui::curses::init();
    let mut user_interface = ui::UserInterface::new(sh);
    user_interface.populate_screen();
    user_interface.mainloop()?;
    ui::curses::teardown();
    Ok(())
}
