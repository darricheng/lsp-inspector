use iced::task::{Sipper, sipper};
use log::{Level, error, info, log};
use std::process::Stdio;
use tokio::io::{self, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Clone, Copy)]
enum StdIoEnum {
    Stdin,
    Stdout,
    // Stderr
}

#[derive(Debug, Clone)]
pub enum LspMessage {
    Client(String),
    Server(String),
}

fn custom_logger(message: &str, level: Level, source: StdIoEnum) {
    log!(level, "-- {:?} -- {}", source, message);
}

async fn extract_message(
    source: impl AsyncRead + std::marker::Unpin,
    mut target: impl AsyncWriteExt + std::marker::Unpin,
    sender: Sender<LspMessage>,
    message_source: StdIoEnum,
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
        custom_logger(
            &format!("Extracted Content-Length: {}", content_length),
            Level::Info,
            message_source,
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
        custom_logger(
            &format!("Extracted message: {}", message),
            Level::Info,
            message_source,
        );

        if (match message_source {
            StdIoEnum::Stdin => sender.send(LspMessage::Client(message)).await,
            StdIoEnum::Stdout => sender.send(LspMessage::Server(message)).await,
        }).is_err() {
            error!("Receiver dropped");
            return;
        };

        // Write to target output
        if target.write_all(&bytes_for_server).await.is_err() {
            break; // output likely closed
        }
        if target.flush().await.is_err() {
            break;
        }
    }
}

pub fn lsp_listener(lsp_command: String) -> impl Sipper<(), LspMessage> {
    sipper(async move |mut output| {
        let (sender, mut receiver) = mpsc::channel::<LspMessage>(16);

        info!("lsp_command: {}", lsp_command);

        let mut child = {
            // NOTE: not sure if this is the best way to build the command
            let mut command_iter = lsp_command.split(' ');
            let mut command = Command::new(command_iter.next().unwrap());
            command_iter.for_each(|arg| {
                command.arg(arg);
            });

            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap()
        };

        let child_stdin = child.stdin.take().unwrap();
        let child_stdout = child.stdout.take().unwrap();

        let stdin_task = tokio::spawn(extract_message(
            io::stdin(),
            child_stdin,
            sender.clone(),
            StdIoEnum::Stdin,
        ));
        let stdout_task = tokio::spawn(extract_message(
            child_stdout,
            io::stdout(),
            sender,
            StdIoEnum::Stdout,
        ));
        let receive_task = tokio::spawn(async move {
            info!("Starting to receive messages");
            while let Some(message) = receiver.recv().await {
                info!("Received message: {:?}", message);
                let _ = output.send(message).await;
            }
        });

        info!("Awaiting tasks");
        let _ = stdin_task.await;
        let _ = stdout_task.await;
        let _ = receive_task.await;

        let status = child
            .wait()
            .await
            .expect("Child process encountered an error");
        log!(Level::Info, "Child process exited with status: {}", status);
    })
}
