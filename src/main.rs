use log::{Level, LevelFilter, error, log};
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::env;
use std::error::Error;
use std::fs::File;
use std::process::Stdio;
use tokio::io::{self, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug)]
enum StdIoEnum {
    Stdin,
    Stdout,
    // Stderr,
}

fn custom_logger(stdio_type: StdIoEnum) -> impl Fn(&str, Level) {
    move |message: &str, level: Level| {
        log!(level, "-- {:?} -- {}", stdio_type, message);
    }
}

async fn extract_message(
    source: impl AsyncRead + std::marker::Unpin,
    mut target: impl AsyncWriteExt + std::marker::Unpin,
    logger: impl Fn(&str, Level),
) {
    let mut buf_reader = BufReader::new(source);
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
        let mut split = content_length_header_str.trim().split(' ');
        split.next(); // Don't need the header name
        let content_length = split.next().unwrap().parse::<usize>().unwrap();
        logger(
            &format!("Extracted Content-Length: {}", content_length),
            Level::Info,
        );

        // Read the next two \r\n bytes
        let mut line_break = String::new();
        buf_reader.read_line(&mut line_break).await.unwrap();

        bytes_for_server.extend_from_slice(line_break.as_bytes());

        // Read the actual content
        let mut message_buf = vec![0; content_length];
        buf_reader.read_exact(&mut message_buf).await.unwrap();

        bytes_for_server.extend_from_slice(&message_buf);

        let message = String::from_utf8(message_buf).unwrap();
        logger(&format!("Extracted message: {}", message), Level::Info);
        // TODO: do something with message

        // Write to child stdin
        if target.write_all(&bytes_for_server).await.is_err() {
            break; // Child stdin likely closed
        }
        if target.flush().await.is_err() {
            break;
        }
    }
}

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
        // TODO: make it possible to pass in the lsp command
        let mut child = Command::new("biome")
            .arg("lsp-proxy")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let child_stdin = child.stdin.take().unwrap();
        let child_stdout = child.stdout.take().unwrap();

        let stdin_task = tokio::spawn(extract_message(
            io::stdin(),
            child_stdin,
            custom_logger(StdIoEnum::Stdin),
        ));
        let stdout_task = tokio::spawn(extract_message(
            child_stdout,
            io::stdout(),
            custom_logger(StdIoEnum::Stdout),
        ));

        let status = child
            .wait()
            .await
            .expect("Child process encountered an error");
        log!(Level::Info, "Child process exited with status: {}", status);

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
