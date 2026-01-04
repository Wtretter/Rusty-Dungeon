use std::str;
use iced::widget::{button, column, text, Column};
use iced::Task;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

const SERVER_ADDR: &str = "rustydungeon.wtretter.com:27010";

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
    ServerConnected(Option<()>),
}

#[derive(Default)]
struct Counter {
    value: i64,
}

async fn connect_to_server() -> Option<()>{
    let mut stream = TcpStream::connect(SERVER_ADDR).await.ok()?;

    let mut size_buffer: [u8; 2] = [0, 0];
    stream.read_exact(&mut size_buffer).await.ok()?;

    let size = u16::from_be_bytes(size_buffer) as usize;

    let mut string_buffer = vec![0; size];
    stream.read_exact(&mut string_buffer).await.ok()?;

    let string = str::from_utf8(&string_buffer).ok()?;
    println!("got message: {}", string);

    return Some(());
}

impl Counter {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                self.value += 1;
                return Task::perform(connect_to_server(), Message::ServerConnected);
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::ServerConnected(Some(_)) => {
                println!("Connection succeeded!");
            }
            Message::ServerConnected(None) => {
                println!("Connection failed!");
            }
        }
        return Task::none();
    }

    fn view(&self) -> Column<'_, Message> {
        // The buttons
        let increment = button("+").on_press(Message::Increment);
        let decrement = button("-").on_press(Message::Decrement);

        // The number
        let counter = text(self.value);

        // The layout
        let interface = column![increment, counter, decrement];

        interface
    }
}

pub fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}