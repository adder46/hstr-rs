use io::print_config;
use structopt::StructOpt;

mod hstr;
mod io;
mod sort;
mod state;
mod ui;

#[derive(Debug, StructOpt)]
struct Opt {
    query: Vec<String>,
    #[structopt(name = "show-config", long)]
    show_config: Option<String>,
}

#[allow(unreachable_code)]
fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();
    if let Some(sh) = opt.show_config {
        print_config(sh.as_str());
        return Ok(());
    }
    let query = opt.query.join(" ");
    let mut user_interface = ui::UserInterface::new(query);
    ui::curses::init();
    user_interface.state.search();
    user_interface.populate_screen();
    user_interface.mainloop()?;
    ui::curses::teardown();

    Ok(())
}
