use iced::task::{Sipper, sipper};
use iced::widget::button;
use iced::{self, Element, Subscription};
use log::{Level, LevelFilter, error, info, log};
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::env;
use std::fs::File;
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
enum LspMessage {
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

        if let Err(_) = match message_source {
            StdIoEnum::Stdin => sender.send(LspMessage::Client(message)).await,
            StdIoEnum::Stdout => sender.send(LspMessage::Server(message)).await,
        } {
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

fn lsp_listener() -> impl Sipper<(), LspMessage> {
    sipper(async |mut output| {
        let (sender, mut receiver) = mpsc::channel::<LspMessage>(16);

        // TODO: make it possible to pass in the lsp command
        let mut child = Command::new("biome")
            .arg("lsp-proxy")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();

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
                let _ = output.send(message);
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

#[derive(Debug, Clone)]
enum Message {
    MessageReceived(LspMessage),
    ButtonPressed,
}

struct LspInspector;

impl LspInspector {
    fn new() -> Self {
        Self
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonPressed => info!("Button Pressed"),
            Message::MessageReceived(source) => {
                info!("Message received in Iced: {:?}", source);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        button("Press me!").on_press(Message::ButtonPressed).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(lsp_listener).map(Message::MessageReceived)
    }
}

fn main() -> iced::Result {
    let current_dir = env::current_dir().unwrap();
    let log_file_path = current_dir.join("lsp-inspector-debug.log");
    let log_file = File::create(log_file_path).unwrap();
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )])
    .unwrap();

    iced::application(LspInspector::new, LspInspector::update, LspInspector::view)
        .subscription(LspInspector::subscription)
        .run()
}
