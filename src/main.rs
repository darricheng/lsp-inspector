use log::{error, info};
use simplelog::*;
use std::env;
use std::error::Error;
use std::fs::File;
use std::process::Stdio;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
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
            let parent_stdin = io::stdin();
            let mut buf_reader = BufReader::new(parent_stdin);
            loop {
                // Buffer for passing content to the server
                // Every time we read from the buffer, we need to add to this slice,
                // so that we don't lose info for the server.
                let mut bytes_for_server: Vec<u8> = Vec::new();

                let mut content_length_header_str = String::new();
                buf_reader
                    .read_line(&mut content_length_header_str)
                    .await
                    .unwrap();

                bytes_for_server.extend_from_slice(content_length_header_str.as_bytes());

                // Extract the content length from the header
                info!("{}", content_length_header_str);
                let mut split = content_length_header_str.trim().split(' ');
                split.next(); // Don't need the header name
                let content_length = split.next().unwrap().parse::<usize>().unwrap();
                info!("Extracted Content-Length: {}", content_length);

                // Read the next two \r\n bytes
                let mut line_break = String::new();
                buf_reader.read_line(&mut line_break).await.unwrap();

                bytes_for_server.extend_from_slice(line_break.as_bytes());

                // Read the actual content
                let mut message_buf = vec![0; content_length];
                buf_reader.read_exact(&mut message_buf).await.unwrap();

                bytes_for_server.extend_from_slice(&message_buf);

                let message = String::from_utf8(message_buf).unwrap();
                info!("Extracted message: {}", message);
                // TODO: do something with message

                // Write to child stdin
                if child_stdin.write_all(&bytes_for_server).await.is_err() {
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
