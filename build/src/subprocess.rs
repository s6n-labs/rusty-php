use std::io::stdout;
use std::process::{Command, Stdio};

use anyhow::bail;

pub fn run(command: &mut Command) -> anyhow::Result<()> {
    let arg0 = command.get_program().to_string_lossy().to_string();
    let mut child = command.stdout(Stdio::piped()).spawn()?;

    std::io::copy(child.stdout.as_mut().unwrap(), &mut stdout()).unwrap();

    let status = child.wait()?;
    if !status.success() {
        bail!(
            "{} failed with exit code {}",
            arg0,
            status
                .code()
                .map(|i| i.to_string())
                .unwrap_or("<unknown>".to_string()),
        );
    }

    Ok(())
}
