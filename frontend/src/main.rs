use std::str;
use iced::widget::{button, scrollable, column, row, text, text_input, Space, Column};
use iced::Task;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::sync::{Arc};
use serde_json::{json};
use serde::{Deserialize, Serialize};
use iced::Length;
use iced::Size;
use iced::window::Settings;

type Connection = Arc<Mutex<TcpStream>>;
const SERVER_ADDR: &str = "rustydungeon.wtretter.com:27010";


#[derive(Default, Clone, Debug, Serialize, Deserialize)]
enum State {
    #[default] Startup=0,
    Login=1,
    Shop=10,
    Run=11,
}

#[derive(Debug, Clone)]
enum Message {
    Startup(()),
    AttemptLogin,
    Register,
    StartRun,
    BuyHP,
    PlayerUpdated(Option<String>),
    RunFinished(Option<String>),
    UsernameChanged(String),
    ServerConnected(Option<Connection>),    
}

#[derive(Default, Serialize, Deserialize)]
struct Player {
    username: String,
	money: u32,
	hitpoints: u32,
	damage: u32,
	luck: u32,
	resistance: u32,
	crit: u32,
    state: State,
}

#[derive(Default)]
struct Application {
    connection: Option<Connection>,
    text: String,
    player: Player,
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

async fn send_message_get_json(conn: Connection, message: serde_json::Value) -> Option<String>{
    {
        let mut stream = conn.lock().await;

        let string = message.to_string();

        let len: u16 = string.len() as u16;
        stream.write_u16(len).await.ok()?;

        let mess_buffer = string.as_bytes();
        stream.write_all(mess_buffer).await.ok()?;
    }

    let message_from_server = recv_message(conn.clone()).await?;
    return Some(message_from_server);
}

impl Application {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                connection: None,
                text: "".to_string(),
                player: Player{
                    username: "".to_string(),
                    money: 0,
                    hitpoints: 0,
                    damage: 0,
                    luck: 0,
                    resistance: 0,
                    crit: 0,
                    state: State::Startup,
                }
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
                            "username": self.player.username.to_string()
                        });
                        return Task::perform(send_message_get_json(conn.clone(), json_message), Message::PlayerUpdated);
                    }
                    None => {
                        println!("No Connection");
                        self.player.state = State::Startup;
                        return Task::done(Message::Startup(()));
                    }
                }
            }
            Message::Register => {
                match &mut self.connection {
                    Some(conn) => {
                        let json_message = json!({
                            "message_type": "register",
                            "username": self.player.username.to_string()
                        });
                        return Task::perform(send_message_get_json(conn.clone(), json_message), Message::PlayerUpdated);
                    }
                    None => {
                        println!("No Connection");
                        self.player.state = State::Startup;
                        return Task::done(Message::Startup(()));
                    }
                }
            }
            Message::StartRun => {
               match &mut self.connection {
                    Some(conn) => {
                        let json_message = json!({
                            "message_type": "run",
                        });
                        return Task::perform(send_message_get_json(conn.clone(), json_message), Message::RunFinished);
                    }
                    None => {
                        println!("No Connection");
                        self.player.state = State::Startup;
                        return Task::done(Message::Startup(()));
                    }
                }
            }
            Message::UsernameChanged(username) => {
                self.player.username = username;
            }
            Message::ServerConnected(Some(conn)) => {
                self.connection = Some(conn);
                println!("Connection succeeded!");
                self.player.state = State::Login;
            }
            Message::ServerConnected(None) => {
                println!("Connection failed!");
                
                self.player.state = State::Startup;
                return Task::perform(sleep(Duration::from_secs(5)), Message::Startup);
            }
            Message::RunFinished(Some(data)) => {
                self.text = data;
                self.player.state = State::Run;
            }
            Message::RunFinished(None) => {
                println!("data failed to send");
                self.player.state = State::Startup;
                return Task::done(Message::Startup(()));
            }
            Message::PlayerUpdated(Some(data)) => {
                println!("{}", data);
                self.player = serde_json::from_str(data.as_str()).expect("server sent invalid player obj");

            }
            Message::PlayerUpdated(None) => {
                println!("no response from server");
                self.player.state = State::Startup;
                return Task::done(Message::Startup(()));
            }
            Message::BuyHP => {
                match &mut self.connection {
                    Some(conn) => {
                        let json_message = json!({
                            "message_type": "buy",
                            "upgrade": "hitpoints"
                        });
                        return Task::perform(send_message_get_json(conn.clone(), json_message), Message::PlayerUpdated);
                    }
                    None => {
                        println!("No Connection");
                        self.player.state = State::Startup;
                        return Task::done(Message::Startup(()));
                    }
                }
            }
        }
        return Task::none();
    }

    fn view(&self) -> Column<'_, Message> {

        match self.player.state {
            State::Login => {
                let username_box = text_input("username", &self.player.username).on_input(Message::UsernameChanged);
                
                let login_button = button("LOGIN").on_press(Message::AttemptLogin);

                let register_button = button("REGISTER").on_press(Message::Register);
                
                let interface = column![username_box, row![login_button, Space::new().width(4), register_button]];
                return interface;
            }

            State::Shop => {
                let run_button = button("Start Run").on_press(Message::StartRun);
                let upgrade_hp_button = button("+Hitpoints").on_press(Message::BuyHP);
                let interface = column![
                    text("Shop"),
                    row![
                        column![
                            text(format!("Funds: {}", self.player.money))
                        ],
                        Space::new().width(Length::Fill),
                        column![
                            upgrade_hp_button
                        ]
                    ],
                    run_button
                ];
                
            
                return interface;    
            }

            State::Run => {
                let return_button = button("Shop").on_press(Message::AttemptLogin);
                let interface = column![text("Run"), Space::new().height(24), scrollable(text(&self.text)).height(500), return_button];

                

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
    iced::application(Application::new, Application::update, Application::view).window(Settings{min_size:Some(Size{width: 80.0, height: 40.0}), ..Default::default()}).run()
}