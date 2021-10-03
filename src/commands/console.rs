use super::super::selection::Selection;

pub fn run(selection: Selection, cmd: Vec<String>) -> anyhow::Result<()> {
    println!(
        "Running console command: {:?} in context {}",
        cmd, selection.context
    );
    Ok(())
}
