use env_logger;
use flightctl::{Config, ConfigFile, Release, Selector};
use log;
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
    /// Run an AWS CLI command for a release
    Aws {
        cmd: Vec<String>,
        #[structopt(flatten)]
        selector: Selector,
    },

    /// Fetch configuration variables for a release
    Config {
        #[structopt(flatten)]
        selector: Selector,
    },

    /// Run a console for a release
    Console {
        #[structopt(flatten)]
        selector: Selector,
    },

    /// Run a kubectl command for a release
    Kubectl {
        cmd: Vec<String>,
        #[structopt(flatten)]
        selector: Selector,
    },

    /// List processes running for a release
    Ps {
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
    log::debug!("Beginning preflight");
    let release = opt.selector.merge(selector).apply(&config)?;
    flightctl::authorize::run(&config, &release)?;
    flightctl::context::prepare(&config, &release)?;
    log::debug!("Preflight complete");
    Ok(release)
}

fn init_logger(default: &str) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default))
        .format_timestamp(None)
        .init();
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let config_file = ConfigFile::find()?;
    let config = config_file.config;

    if opt.debug {
        init_logger("debug");
    } else {
        init_logger("info");
    }

    if log::log_enabled!(log::Level::Debug) {
        log::debug!("Loaded configuration from {}:", config_file.path.display());
        log::debug!("{:?}", config);
        log::debug!("Parsed command: {:?}", opt.cmd);
    }

    match opt.cmd {
        Some(Command::Aws {
            ref cmd,
            ref selector,
        }) => {
            let release = preflight(&config, &opt, &selector)?;
            commands::aws::run(&config, release, cmd)
        }
        Some(Command::Config { ref selector }) => {
            let release = preflight(&config, &opt, &selector)?;
            commands::config::print(&config, release)
        }
        Some(Command::Console { ref selector }) => {
            let release = preflight(&config, &opt, &selector)?;
            commands::console::run_default(&config, release)
        }
        Some(Command::Kubectl {
            ref cmd,
            ref selector,
        }) => {
            let release = preflight(&config, &opt, &selector)?;
            commands::kubectl::run(&config, release, cmd)
        }
        Some(Command::Ps { ref selector }) => {
            let release = preflight(&config, &opt, &selector)?;
            commands::process::run(&config, release)
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
