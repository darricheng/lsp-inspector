use std::io;
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    // Configure the child process's stdin to inherit from the parent.
    let mut child = Command::new("biome")
        .arg("lsp-proxy")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // The parent process can then wait for the child to finish.
    let status = child.wait()?;
    println!("Child process exited with status: {}", status);

    Ok(())
}
