use config::Config;
use selector::Selector;
use structopt::StructOpt;

mod commands;
mod config;
mod selection;
mod selector;

#[derive(Debug, StructOpt)]
#[structopt(name = "flightctl", about = "control a cloud workspace")]
struct Opt {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(subcommand)]
    cmd: Option<Command>,

    #[structopt(flatten)]
    selector: Selector,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Run a console command for a release
    Console {
        cmd: Vec<String>,

        #[structopt(flatten)]
        selector: Selector,
    },

    /// View information about this workspace
    View {
        #[structopt(subcommand)]
        cmd: ViewCommand,
    },
}

#[derive(Debug, StructOpt)]
enum ViewCommand {
    /// View applications for this workspace
    Applications,

    /// View authorization for this workspace
    Auth,

    /// View clusters for this workspace
    Clusters,

    /// View contexts for this workspace
    Contexts,

    /// View releases for this workspace
    Releases,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let config_file = Config::find()?;
    let config = config_file.config;

    if opt.debug {
        println!("Loaded configuration from {}:", config_file.path.display());
        println!("{:?}", config);
        println!("Running command: {:?}", opt.cmd);
    }

    match opt.cmd {
        Some(Command::Console { cmd, selector }) => {
            let selection = opt.selector.merge(selector).apply(&config)?;
            commands::console::run(selection, cmd)
        }
        Some(Command::View {
            cmd: ViewCommand::Applications,
        }) => commands::view::applications(config),
        Some(Command::View {
            cmd: ViewCommand::Auth,
        }) => commands::view::auth(config),
        Some(Command::View {
            cmd: ViewCommand::Clusters,
        }) => commands::view::clusters(config),
        Some(Command::View {
            cmd: ViewCommand::Contexts,
        }) => commands::view::contexts(config),
        Some(Command::View {
            cmd: ViewCommand::Releases,
        }) => commands::view::releases(config),
        None => {
            Opt::clap().print_help()?;
            println!("");
            Ok(())
        }
    }
}
