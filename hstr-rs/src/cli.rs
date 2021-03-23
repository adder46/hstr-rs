use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    pub query: Vec<String>,
    #[structopt(name = "show-config", long)]
    pub show_config: Option<String>,
}
