use log::{error, info};
use simplelog::*;
use std::env;
use std::error::Error;
use std::fs::File;
use std::process::Stdio;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

fn main() {
    let current_dir = env::current_dir().unwrap();
    let log_file_path = current_dir.join("lsp-inspector-debug.log");
    let log_file = File::create(log_file_path).unwrap();
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )])
    .unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let result: Result<(), Box<dyn Error>> = rt.block_on(async {
        // Configure the child process's stdin to inherit from the parent.
        let mut child = Command::new("biome")
            .arg("lsp-proxy")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut child_stdin = child.stdin.take().unwrap();
        let child_stdout = child.stdout.take().unwrap();

        let stdin_task = tokio::spawn(async move {
            let mut parent_stdin = io::stdin();
            let mut buffer = vec![0; 1024];
            loop {
                // Read from parent stdin
                let n = match parent_stdin.read(&mut buffer).await {
                    Ok(0) => break, // Reached EOF
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Error reading from parent stdin: {}", e);
                        break;
                    }
                };

                let mut copied = vec![0; n];
                info!("after init copied");
                copied.copy_from_slice(&buffer);
                info!("copied into buffer");
                let copied_str = String::from_utf8(copied).unwrap();
                info!("copied_str: {}", copied_str);

                // Write to child stdin
                if child_stdin.write_all(&buffer[..n]).await.is_err() {
                    break; // Child stdin likely closed
                }
                if child_stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        let stdout_task = tokio::spawn(async move {
            let mut parent_stdout = io::stdout();
            let mut reader = BufReader::new(child_stdout);
            let mut buffer = vec![0; 1024];
            loop {
                // Read from child stdout
                let n = match reader.read(&mut buffer).await {
                    Ok(0) => break, // Reached EOF
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Error reading from child stdout: {}", e);
                        break;
                    }
                };
                // Write to parent stdout
                if parent_stdout.write_all(&buffer[..n]).await.is_err() {
                    break; // Parent stdout likely closed
                }
                if parent_stdout.flush().await.is_err() {
                    break;
                }
            }
        });

        let status = child
            .wait()
            .await
            .expect("Child process encountered an error");
        println!("Child process exited with status: {}", status);

        let _ = stdin_task.await;
        let _ = stdout_task.await;

        Ok(())
    });

    match result {
        Ok(_) => (),
        Err(e) => {
            error!("Failed: {:?}", e.to_string());
        }
    }
}
