use crate::flightctl::aws;
use crate::flightctl::{AuthConfig, Config, Release};

pub fn run(config: &Config, release: &Release, cmd: &Vec<String>) -> anyhow::Result<()> {
    let context = config.find_context(release)?;
    let auth = config.find_auth(context)?;

    match &auth.config {
        AuthConfig::AwsSso { .. } => aws::run_cli_print(
            &[
                vec!["--profile", &auth.name],
                cmd.iter().map(|s| s.as_ref()).collect(),
            ]
            .concat(),
        ),
    }
}
