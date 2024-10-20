use std::{
    env,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use iced::{
    color, executor,
    theme::Theme,
    widget::{button, column, container, horizontal_space, row, text, text_editor, Container},
    Application, Command, Element, Length, Settings,
};
use rfd::AsyncFileDialog;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
};
use tokio::{fs::read_to_string, time};

fn main() -> iced::Result {
    SolanaProgram::run(Settings::default())
}

struct SolanaProgram {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
    balance: Option<u64>,
}

#[derive(Debug, Clone)]
enum Message {
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Open,
    BalanceLoaded(Result<u64, Error>),
    ErrorCleared,
}

const DEFAULT_LOCATION: &str = ".config/solana/id.json";

const RPC_URL: &str = "https://api.devnet.solana.com";

impl Application for SolanaProgram {
    type Message = Message;

    type Executor = executor::Default;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
                balance: None,
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("Solana Executable Application")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path.to_path_buf());
                self.content = text_editor::Content::with(&content);
                Command::perform(display_balance(path.to_path_buf()), Message::BalanceLoaded)
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::perform(async { time::sleep(Duration::from_secs(5)).await }, |_| {
                    Message::ErrorCleared
                })
            }
            Message::BalanceLoaded(Ok(balance)) => {
                self.balance = Some(balance);
                Command::none()
            }
            Message::BalanceLoaded(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::ErrorCleared => {
                self.error = None;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![button("Load keypair").on_press(Message::Open)];

        // get the file_path
        let file_path = self
            .path
            .as_deref()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(""));

        // display the publickey of the keypair
        let display_pkey = display_pubkey(file_path.to_path_buf());

        // display the SOL balance
        let balance_text = match self.balance {
            Some(balance) => row![
                text("SOL Balance: ").style(color!(0x30cbf2)).size(14),
                text(format!(" {:.3}", balance as f32 / LAMPORTS_PER_SOL as f32)).size(14)
            ],
            None => row![text("Loading balance...").size(14)],
        };

        let info_message = if let Some(ref error) = &self.error {
            text(format!("Error: {:?}", error))
                .size(14)
                .style(color!(0xFF0000))
        } else {
            text("").size(1)
        };

        let file_path_name = text(file_path.to_str().unwrap_or(DEFAULT_LOCATION)).size(14);
        let file_path_indicator = text("Path of your keypair:")
            .size(14)
            .style(color!(0x30cbf2));

        let status_bar = row![file_path_name, horizontal_space(Length::Fixed(100.0)),];

        container(
            column![
                display_pkey,
                balance_text,
                file_path_indicator,
                status_bar,
                controls,
                info_message,
            ]
            .spacing(10),
        )
        .padding(25)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn display_pubkey(file_path: PathBuf) -> Element<'static, Message> {
    let keypair = load_keypair_from_file(file_path);

    let public_key_display = text(format!("Wallet address: {}", keypair.pubkey().to_string()))
        .size(14)
        .width(Length::Fixed(400.0))
        .height(Length::Shrink);

    let pubkey_container = Container::new(public_key_display).width(Length::Fixed(400.0));
    pubkey_container.into()
}

async fn display_balance(path: PathBuf) -> Result<u64, Error> {
    let keypair = load_keypair_from_file(path);
    let client = Arc::new(RpcClient::new(RPC_URL.to_string()));
    client
        .get_balance(&keypair.pubkey())
        .await
        .map_err(|_| Error::FetchBalanceError)
}

fn load_keypair_from_file(path: PathBuf) -> Keypair {
    let keypair = read_keypair_file(path).unwrap_or(Keypair::new());
    keypair
}

fn default_file() -> PathBuf {
    let home_dir = env::var("HOME") // mac users
        .or_else(|_| env::var("USERPROFILE")) // windows users
        .expect("Cannot find home directory");
    let mut path = PathBuf::from(home_dir);
    path.push(DEFAULT_LOCATION);
    path
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = AsyncFileDialog::new()
        .set_title("Choose a valid json solana keypair")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    if handle.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Err(Error::InvalidFileType);
    }
    load_file(handle.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|err| err.kind())
        .map_err(Error::IO)?;
    Ok((path, contents))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(ErrorKind),
    FetchBalanceError,
    InvalidFileType,
}
