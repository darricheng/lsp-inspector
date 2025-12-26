use crate::lsp::{LspMessage, lsp_listener};
use iced::Length::Fill;
use iced::widget::{Container, Grid, button, column, container, scrollable, space, text};
use iced::{Element, Subscription};
use log::info;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum Message {
    MessageReceived(LspMessage),
    SetShownMessageId(usize),
}

pub struct LspInspector {
    lsp_messages: Vec<LspMessage>,
    selected_message_index: Option<usize>,
}

impl LspInspector {
    pub fn new() -> Self {
        Self {
            lsp_messages: Vec::new(),
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
            Message::SetShownMessageId(id) => {
                self.selected_message_index = Some(id);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let shown_message: Element<_> = {
            let msg_enum = self
                .selected_message_index
                .map(|id| self.lsp_messages.get(id).unwrap());
            let msg: Element<_> = match msg_enum {
                Some(lsp_msg) => match lsp_msg {
                    LspMessage::Client(m) => text(m).into(),
                    LspMessage::Server(m) => text(m).into(),
                },
                None => space().into(),
            };
            msg
        };

        let message_elements: Element<_> = scrollable(column(
            self.lsp_messages
                .iter()
                .enumerate()
                .map(|(i, msg)| -> Container<'_, Message> {
                    match msg {
                        LspMessage::Client(m) => container(
                            button(text(summarise_message(m)).width(200))
                                .on_press(Message::SetShownMessageId(i)),
                        )
                        .align_left(Fill),
                        LspMessage::Server(m) => container(
                            button(text(summarise_message(m)).width(200))
                                .on_press(Message::SetShownMessageId(i)),
                        )
                        .align_right(Fill),
                    }
                })
                .map(Element::from),
        ))
        .width(Fill)
        .into();

        Grid::with_children([shown_message, message_elements])
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
                .map(Message::MessageReceived)
        }
    }
}

// Extract the combination of id and method.
fn summarise_message(json_str: &str) -> String {
    let v: Value = serde_json::from_str(json_str).unwrap();
    let id = v["id"].as_number();
    let method = v["method"].as_str();

    // Request: id & method
    // Response: id only
    // Notification: method only
    let res = match (id, method) {
        (Some(i), Some(m)) => format!("Request\nid: {}\nmethod: {}", i, m),
        (Some(i), None) => format!("Response\nid: {}", i),
        (None, Some(m)) => format!("Notification\nmethod: {}", m),
        (None, None) => "ERROR: Unknown message type".to_owned(),
    };

    info!(
        "id: {:?}, method: {:?}, res: {:?}, v: {:?}",
        id, method, res, v
    );
    res
}
