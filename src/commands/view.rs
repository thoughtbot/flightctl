use super::super::config::Config;

pub fn applications(config: Config) -> anyhow::Result<()> {
    for application in config.applications {
        println!("{}", application);
    }

    Ok(())
}

pub fn auth(config: Config) -> anyhow::Result<()> {
    for auth in config.auth {
        println!("{}", auth);
    }

    Ok(())
}

pub fn clusters(config: Config) -> anyhow::Result<()> {
    for cluster in config.clusters {
        println!("{}", cluster);
    }

    Ok(())
}

pub fn contexts(config: Config) -> anyhow::Result<()> {
    for context in config.contexts {
        println!("{}", context);
    }

    Ok(())
}

pub fn releases(config: Config) -> anyhow::Result<()> {
    for release in config.releases {
        println!("{}", release);
    }

    Ok(())
}
