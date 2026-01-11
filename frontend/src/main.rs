use std::str;
use iced::widget::{button, column, text, Column};
use iced::Task;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::{Arc};

type Connection = Arc<Mutex<TcpStream>>;
const SERVER_ADDR: &str = "rustydungeon.wtretter.com:27010";

#[derive(Debug, Clone)]
enum Message {
    Startup,
    Increment,
    Decrement,
    ServerConnected(Option<Connection>),
    DataSent(Option<()>),
}

#[derive(Default)]
struct Application {
    value: i64,
    connection: Option<Connection>,
}

async fn initialize() -> Message {
    return Message::Startup;
}

async fn connect_to_server() -> Option<Connection>{
    let conn = Arc::new(Mutex::new(TcpStream::connect(SERVER_ADDR).await.ok()?));
    return Some(conn);
}

async fn recv_message(conn: Connection) -> Option<String>{
    let mut stream = conn.lock().await;

    let mut size_buffer: [u8; 2] = [0, 0];
    stream.read_exact(&mut size_buffer).await.ok()?;

    let size = u16::from_be_bytes(size_buffer) as usize;

    let mut string_buffer = vec![0; size];
    stream.read_exact(&mut string_buffer).await.ok()?;

    let message = str::from_utf8(&string_buffer).ok()?;
    return Some(message.to_string());
}

async fn send_message(conn: Connection, string: String) -> Option<()>{
    {
        let mut stream = conn.lock().await;

        let len: u16 = string.len() as u16;
        stream.write_u16(len).await.ok()?;

        let mess_buffer = string.as_bytes();
        stream.write_all(mess_buffer).await.ok()?;
    }

    let mess_recv = recv_message(conn.clone()).await?;
    println!("{}", mess_recv);

    return Some(());
}

impl Application {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                value: 0,
                connection: None,
            },
            Task::future(initialize())
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        
        match message {
            Message::Startup => {
                println!("Starting up!");
                return Task::perform(connect_to_server(), Message::ServerConnected);
            }
            Message::Increment => {
                self.value += 1;
                match &mut self.connection {
                    Some(conn) => {
                        return Task::perform(send_message(conn.clone(), "asdf".to_string()), Message::DataSent);
                    }
                    None => {
                        println!("No Connection")
                    }
                }
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::ServerConnected(Some(conn)) => {
                self.connection = Some(conn);
                println!("Connection succeeded!");
            }
            Message::ServerConnected(None) => {
                println!("Connection failed!");
            }
            Message::DataSent(Some(_)) => {}
            
            Message::DataSent(None) => {
                println!("data failed to send");
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

        return interface;
    }
}

pub fn main() -> iced::Result {
    iced::application(Application::new, Application::update, Application::view).run()
}