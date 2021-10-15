use super::aws;
use super::config::{Auth, AuthConfig, Config, Release};

pub fn run(config: &Config, release: &Release) -> anyhow::Result<()> {
    let context = config.find_context(&release)?;
    let auth = config.find_auth(context)?;
    ensure_auth(auth)
}

fn ensure_auth(auth: &Auth) -> anyhow::Result<()> {
    match &auth.config {
        AuthConfig::AwsSso { config: sso_config } => {
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
