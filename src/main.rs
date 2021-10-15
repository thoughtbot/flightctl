use flightctl::{Config, ConfigFile, Release, Selector};
use structopt::StructOpt;

mod commands;
mod flightctl;

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
    /// Run a console for a release
    Console {
        #[structopt(flatten)]
        selector: Selector,
    },

    /// Run a container command for a release
    Run {
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

fn preflight<'a>(
    config: &'a Config,
    opt: &Opt,
    selector: &Selector,
) -> anyhow::Result<&'a Release> {
    let release = opt.selector.merge(selector).apply(&config)?;
    flightctl::authorize::run(&config, &release)?;
    flightctl::context::prepare(&config, &release)?;
    Ok(release)
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let config_file = ConfigFile::find()?;
    let config = config_file.config;

    if opt.debug {
        println!("Loaded configuration from {}:", config_file.path.display());
        println!("{:?}", config);
        println!("Running command: {:?}", opt.cmd);
    }

    match opt.cmd {
        Some(Command::Console { ref selector }) => {
            let release = preflight(&config, &opt, &selector)?;
            commands::console::run_default(&config, release)
        }
        Some(Command::Run {
            ref cmd,
            ref selector,
        }) => {
            let release = preflight(&config, &opt, &selector)?;
            commands::console::run_command(&config, release, cmd)
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
