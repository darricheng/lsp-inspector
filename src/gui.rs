use crate::lsp::{LspMessage, lsp_listener};
use iced::Length::Fill;
use iced::widget::{Container, column, container, scrollable, text};
use iced::{Element, Subscription};
use log::info;

#[derive(Debug, Clone)]
pub enum Message {
    MessageReceived(LspMessage),
}

pub struct LspInspector {
    lsp_messages: Vec<LspMessage>,
    shown_message: Option<String>,
    selected_message_index: Option<usize>,
}

impl LspInspector {
    pub fn new() -> Self {
        Self {
            lsp_messages: Vec::new(),
            shown_message: None,
            selected_message_index: None,
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
        let message_elements: Element<_> = scrollable(column(
            self.lsp_messages
                .iter()
                .map(|msg| -> Container<'_, Message> {
                    match msg {
                        LspMessage::Client(m) => container(text(m).width(200)).align_left(Fill),
                        LspMessage::Server(m) => container(text(m).width(200)).align_right(Fill),
                    }
                })
                .map(Element::from),
        ))
        .width(Fill)
        .into();

        message_elements
    }

    pub fn subscription(lsp_command: String) -> impl Fn(&Self) -> Subscription<Message> {
        // We pass the returned fn to iced's subscription builder which requires that fn
        // accepts &Self, but it's not used here, so we add an underscore to it.
        move |_lsp_inspector| {
            // Is there a better way than doing all this cloning?
            Subscription::run_with(lsp_command.clone(), |data| lsp_listener(data.clone()))
                .map(Message::MessageReceived)
        }
    }
}
