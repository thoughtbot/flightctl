use super::config::{Application, Config, Release};
use super::selection::Selection;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Selector {
    #[structopt(short, long)]
    pub application: Option<String>,

    #[structopt(short, long)]
    pub environment: Option<String>,
}

impl Selector {
    pub fn apply<'a>(self, config: &'a Config) -> anyhow::Result<Selection<'a>> {
        let context = self.context(&config)?;
        Ok(Selection { context: context })
    }

    pub fn merge(self, other: Selector) -> Selector {
        Selector {
            application: other.application.or(self.application),
            environment: other.environment.or(self.environment),
        }
    }

    fn application<'a>(&self, config: &'a Config) -> anyhow::Result<&'a Application> {
        match &self.application {
            Some(name) => {
                for application in &config.applications {
                    if &application.name == name {
                        return Ok(application);
                    }
                }
                Err(anyhow::Error::msg("No application found by that name"))
            }
            None => {
                if config.applications.len() == 1 {
                    Ok(&config.applications[0])
                } else {
                    Err(anyhow::Error::msg("More than one application"))
                }
            }
        }
    }

    fn release<'a>(&self, config: &'a Config) -> anyhow::Result<&'a Release> {
        let application = self.application(&config)?;

        match &self.environment {
            Some(name) => {
                for release in &config.releases {
                    if &release.environment == name && release.application == application.name {
                        return Ok(release);
                    }
                }
                Err(anyhow::Error::msg(
                    "No release found for that application and environment",
                ))
            }
            None => {
                if config.releases.len() == 1 {
                    Ok(&config.releases[0])
                } else {
                    Err(anyhow::Error::msg(format!(
                        "More than one release for application {}",
                        application.name,
                    )))
                }
            }
        }
    }

    fn context<'a>(&self, config: &'a Config) -> anyhow::Result<&'a String> {
        let release = self.release(&config)?;
        Ok(&release.context)
    }
}
