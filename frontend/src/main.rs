use std::str;
use iced::widget::{button, column, text, text_input, Column};
use iced::Task;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::sync::{Arc};
use serde_json::{Result, Value, json};

type Connection = Arc<Mutex<TcpStream>>;
const SERVER_ADDR: &str = "rustydungeon.wtretter.com:27010";

#[derive(Debug, Clone)]
enum Message {
    Startup(()),
    Increment,
    Decrement,
    AttemptLogin,
    UsernameChanged(String),
    ServerConnected(Option<Connection>),
    DataSent(Option<()>),
}

#[derive(Default)]
enum State {
    #[default] Startup=0000,
    Login=0001,
    Shop=0010,
    Run=0011,
}

#[derive(Default)]
struct Application {
    value: i64,
    connection: Option<Connection>,
    state: State,
    username: String,
}

async fn initialize() -> Message {
    return Message::Startup(());
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

async fn recv_uint16(conn: Connection) -> Option<u16>{
    let mut stream = conn.lock().await;

    let mut buffer: [u8; 2] = [0, 0];
    stream.read_exact(&mut buffer).await.ok()?;

    let uint16 = u16::from_be_bytes(buffer);

    return Some(uint16);
}

async fn send_message(conn: Connection, message: serde_json::Value) -> Option<()>{
    {
        let mut stream = conn.lock().await;

        let string = message.to_string();

        let len: u16 = string.len() as u16;
        stream.write_u16(len).await.ok()?;

        let mess_buffer = string.as_bytes();
        stream.write_all(mess_buffer).await.ok()?;
    }

    let mess_recv = recv_uint16(conn.clone()).await?;
    println!("{}", mess_recv);

    return Some(());
}

impl Application {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                value: 0,
                connection: None,
                state: State::Startup,
                username: "".to_string(),
            },
            Task::future(initialize())
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        
        match message {
            Message::Startup(_) => {
                println!("Starting up!");
                return Task::perform(connect_to_server(), Message::ServerConnected);
            }
            Message::AttemptLogin => {
                match &mut self.connection {
                    Some(conn) => {
                        let json_message = json!({
                            "message_type": "login",
                            "username": self.username.to_string()
                        });
                        return Task::perform(send_message(conn.clone(), json_message), Message::DataSent);
                    }
                    None => {
                        println!("No Connection");
                        self.state = State::Startup;
                        return Task::done(Message::Startup(()));
                    }
                }            }
            Message::Increment => {
                self.value += 1;
                match &mut self.connection {
                    Some(conn) => {
                        
                    }
                    None => {
                        println!("No Connection")
                    }
                }
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::UsernameChanged(username) => {
                self.username = username;
            }
            Message::ServerConnected(Some(conn)) => {
                self.connection = Some(conn);
                println!("Connection succeeded!");
                self.state = State::Login;
            }
            Message::ServerConnected(None) => {
                println!("Connection failed!");
                
                self.state = State::Startup;
                return Task::perform(sleep(Duration::from_secs(5)), Message::Startup);
            }
            Message::DataSent(Some(_)) => {}
            
            Message::DataSent(None) => {
                println!("data failed to send");
                self.state = State::Startup;
                return Task::done(Message::Startup(()));
            }
        }
        return Task::none();
    }

    fn view(&self) -> Column<'_, Message> {
        // // The buttons
        // let increment = button("+").on_press(Message::Increment);
        // let decrement = button("-").on_press(Message::Decrement);

        // // The number
        // let counter = text(self.value);
        // let username

        // // The layout
        match self.state {
            State::Login => {
                let username_box = text_input("username", &self.username).on_input(Message::UsernameChanged);
                
                let login_button = button("LOGIN").on_press(Message::AttemptLogin);
                
                let interface = column![username_box, login_button];
                return interface;
            }
            _ => {
                let interface = column![text("error")];
                return interface;
            }
        }
    }
}

pub fn main() -> iced::Result {
    iced::application(Application::new, Application::update, Application::view).run()
}