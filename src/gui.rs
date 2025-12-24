use crate::lsp::{LspMessage, lsp_listener};
use iced::Length::Fill;
use iced::widget::{Grid, column, scrollable, text};
use iced::{Element, Subscription};
use log::info;

#[derive(Debug, Clone)]
pub enum Message {
    MessageReceived(LspMessage),
}

pub struct LspInspector {
    lsp_messages: Vec<LspMessage>,
}

impl LspInspector {
    pub fn new() -> Self {
        Self {
            lsp_messages: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        info!("Update received: {:?}", message);
        match message {
            Message::MessageReceived(message) => {
                info!("Message received in Iced: {:?}", message);
                self.lsp_messages.push(message)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let (client_messages, server_messages): (Vec<_>, Vec<_>) = self
            .lsp_messages
            .iter()
            .partition(|msg| matches!(msg, LspMessage::Client(_)));

        let client_element: Element<_> = scrollable(
            column(
                client_messages
                    .iter()
                    .map(|e| {
                        let msg = if let LspMessage::Client(m) = e {
                            m.to_owned()
                        } else {
                            String::from(
                                "Error: shouldn't have server message in client messages vec",
                            )
                        };
                        text(msg)
                    })
                    .map(Element::from),
            )
            .spacing(10),
        )
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .into();
        let server_element: Element<_> = scrollable(
            column(
                server_messages
                    .iter()
                    .map(|e| {
                        let msg = if let LspMessage::Server(m) = e {
                            m.to_owned()
                        } else {
                            String::from(
                                "Error: shouldn't have client message in server messages vec",
                            )
                        };
                        text(msg)
                    })
                    .map(Element::from),
            )
            .spacing(10),
        )
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .into();

        Grid::with_children([client_element, server_element])
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
