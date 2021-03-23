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
    match opt.show_config {
        Some(sh) => {
            print_config(sh.as_str());
            std::process::exit(0);
        }
        None => {}
    }
    ui::curses::init();
    let processed_query = opt.query.join(" ");
    let mut user_interface = ui::UserInterface::new(processed_query);
    user_interface.app.search();
    user_interface.populate_screen();
    user_interface.mainloop()?;
    ui::curses::teardown();
    Ok(())
}
