use io::print_config;
use structopt::StructOpt;

mod app;
mod hstr;
mod io;
mod sort;
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
    ui::curses::init();
    let query = opt.query.join(" ");
    let mut user_interface = ui::UserInterface::new(query);
    user_interface.app.search();
    user_interface.populate_screen();
    user_interface.mainloop()?;
    ui::curses::teardown();

    Ok(())
}
