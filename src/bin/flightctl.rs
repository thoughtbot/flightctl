use flightctl::commands::{listen, ping, version};

use clap::{Arg, Command};

fn cli() -> Command {
    Command::new("flightctl")
        .about("A command line tool for managing Flightdeck.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("config").about("TODO"))
        .subcommand(Command::new("console").about("TODO"))
        .subcommand(Command::new("help").about("TODO"))
        .subcommand(Command::new("kubctl").about("TODO"))
        .subcommand(Command::new("ps").about("TODO"))
        .subcommand(Command::new("run").about("TODO"))
        .subcommand(Command::new("view").about("TODO"))

/*
        .subcommand(
            Command::new("ping")
                .about("Pings an IP address")
                .arg(Arg::new("ip").required(true).help("IP address to ping")),
        )
        */
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("config", _sub_matches)) => { let _ = config().await; }
        Some(("console", _sub_matches)) => { let _ = console().await; }
        Some(("help", _sub_matches)) => { let _ = help().await; }
        Some(("kubectl", _sub_matches)) => { let _ = kubectl().await; }
        Some(("ps", _sub_matches)) => { let _ = ps().await; }
        Some(("run", _sub_matches)) => { let _ = run().await; }
        Some(("view", _sub_matches)) => { let _ = view().await; }

        /*
        Some(("ping", _sub_matches)) => {
            let ip = _sub_matches.get_one::<String>("ip").unwrap();
            ping(&ip).await?;
        }
        */

        _ => unreachable!(),
    }

    Ok(())
}
