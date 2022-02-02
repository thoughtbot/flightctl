use super::aws;
use super::config::{Auth, AuthConfig, Config, Release};
use log;

pub fn run(config: &Config, release: &Release) -> anyhow::Result<()> {
    log::debug!("Beginning authorization");
    if log::log_enabled!(log::Level::Debug) {
        log::debug!("Checking configured auth for release: {:?}", release);
    }
    let context = config.find_context(&release)?;
    if log::log_enabled!(log::Level::Debug) {
        log::debug!("Found context: {:?}", context);
    }
    let auth = config.find_auth(context)?;
    if log::log_enabled!(log::Level::Debug) {
        log::debug!("Found auth: {:?}", auth);
    }
    ensure_auth(auth)?;
    log::debug!("Authorization successful");
    Ok(())
}

fn ensure_auth(auth: &Auth) -> anyhow::Result<()> {
    match &auth.config {
        AuthConfig::AwsSso { config: sso_config } => {
            log::info!("Authorizing using AWS SSO");
            ensure_aws_profile(&auth.name, || aws::create_profile(&auth.name, sso_config))?;
            aws::verify_auth(&auth.name).or_else(|_| {
                aws::sso_login(&auth.name)?;
                aws::verify_auth(&auth.name)
            })
        }
    }
}

fn ensure_aws_profile<F>(name: &String, or: F) -> anyhow::Result<()>
where
    F: FnOnce() -> anyhow::Result<()>,
{
    let profile_exists = aws::profile_exists(name)?;

    if profile_exists {
        Ok(())
    } else {
        or()
    }
}
