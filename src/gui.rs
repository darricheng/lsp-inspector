use crate::lsp::{LspMessage, lsp_listener};
use iced::Length::Fill;
use iced::widget::{column, container, scrollable, text};
use iced::{self, Color, Element, Subscription};
use log::info;

#[derive(Debug, Clone)]
pub enum Message {
    MessageReceived(LspMessage),
}

pub struct LspInspector {
    client_messages: Vec<String>,
    server_messages: Vec<String>,
}

impl LspInspector {
    pub fn new() -> Self {
        Self {
            client_messages: vec![String::from("Client Messages")],
            server_messages: vec![String::from("Server Messages")],
        }
    }

    pub fn update(&mut self, message: Message) {
        info!("Update received: {:?}", message);
        match message {
            Message::MessageReceived(message) => {
                info!("Message received in Iced: {:?}", message);
                match message {
                    LspMessage::Client(c) => self.client_messages.push(c),
                    LspMessage::Server(s) => self.server_messages.push(s),
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let client_messages: Element<_> = container(
            scrollable(
                column(self.client_messages.iter().map(text).map(Element::from)).spacing(10),
            )
            .height(Fill)
            .spacing(10),
        )
        .style(|_theme| container::Style::default().background(Color::BLACK.scale_alpha(0.8)))
        .into();
        let server_messages: Element<_> = scrollable(
            column(self.server_messages.iter().map(text).map(Element::from)).spacing(10),
        )
        .height(Fill)
        .spacing(10)
        .into();

        column![client_messages, server_messages].into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(lsp_listener).map(|msg| {
            info!("In subscription map");
            Message::MessageReceived(msg)
        })
    }
}
