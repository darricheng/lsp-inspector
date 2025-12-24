use crate::lsp::{LspMessage, lsp_listener};
use iced::Length::Fill;
use iced::widget::{Grid, Scrollable, column, container, grid, scrollable, text};
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
        let client_messages: Element<_> = scrollable(
            column(self.client_messages.iter().map(text).map(Element::from)).spacing(10),
        )
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .into();
        let server_messages: Element<_> = scrollable(
            column(self.server_messages.iter().map(text).map(Element::from)).spacing(10),
        )
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .into();

        Grid::with_children([client_messages, server_messages])
            .columns(2)
            .height(Fill)
            .into()
    }

    pub fn subscription(lsp_command: String) -> impl Fn(&Self) -> Subscription<Message> {
        // We pass the returned fn to iced's subscription builder which requires that fn
        // accepts &Self, but it's not used here, so we add an underscore to it.
        move |_lsp_inspector| {
            // Is there a better way than doing all this cloning?
            Subscription::run_with(lsp_command.clone(), |data| lsp_listener(data.clone()))
                .map(|msg| Message::MessageReceived(msg))
        }
    }
}
