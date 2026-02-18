use crate::cli::{Cli, Shell};

pub fn completions(shell: Shell) -> anyhow::Result<()> {
    use clap::CommandFactory;
    use clap_complete::generate;

    let mut cmd = Cli::command();
    let shell = shell.to_clap_shell();

    generate(shell, &mut cmd, "pebbles", &mut std::io::stdout());

    Ok(())
}
