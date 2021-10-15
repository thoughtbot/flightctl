use crate::flightctl::Release;

pub fn run(release: &Release, cmd: Vec<String>) -> anyhow::Result<()> {
    println!(
        "Running console command: {:?} in context {}",
        cmd, release.context
    );
    Ok(())
}
