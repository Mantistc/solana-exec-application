use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use iced::{
    color, executor,
    theme::Theme,
    widget::{button, column, container, row, text, text_input, Column, Image, Space},
    Application, Command, Element, Settings, Subscription,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair};
use tokio::time;
mod errors;
mod files;
mod loaders;
mod transaction;

use errors::Error;
use files::{default_file, pick_file, DEFAULT_LOCATION};
use loaders::{display_balance, display_pubkey, load_keypair_from_file};
use transaction::transfer_sol;

fn main() -> iced::Result {
    SolExecApp::run(Settings::default())
}

struct SolExecApp {
    pub signer: Arc<Keypair>,
    pub rpc_client: Arc<RpcClient>,
    pub path: Option<PathBuf>,
    pub error: Option<Error>,
    pub balance: Option<u64>,
    pub receiver_value: (String, String),
    pub signature: String,
    pub is_loading: bool,
    pub current_frame: usize,
}

#[derive(Debug, Clone)]
enum Message {
    FileOpened(Result<PathBuf, Error>),
    Open,
    BalanceLoaded(Result<u64, Error>),
    ErrorCleared,
    TxValuesHandler((String, String)),
    ExecuteTransaction,
    TransactionExecuted(Result<String, Error>),
    // for ./gif_animation/loader animation
    NextFrame,
}

const RPC_URL: &str = "https://api.devnet.solana.com";

impl Application for SolExecApp {
    type Message = Message;

    type Executor = executor::Default;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: Some(default_file()),
                error: None,
                balance: None,
                rpc_client: Arc::new(RpcClient::new(RPC_URL.to_string())),
                signer: Keypair::new().into(),
                receiver_value: (String::new(), String::new()),
                signature: String::new(),
                is_loading: false,
                current_frame: 0,
            },
            Command::perform(async { Ok(default_file()) }, Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("Solana Executable Application")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::FileOpened(Ok(path)) => {
                self.path = Some(path.to_path_buf());
                self.signer = load_keypair_from_file(path.to_path_buf()).into();
                Command::perform(
                    display_balance(path, self.rpc_client.clone()),
                    Message::BalanceLoaded,
                )
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
            Message::ExecuteTransaction => {
                self.signature = String::new();
                self.is_loading = true;
                let values = SolExecApp {
                    signer: Arc::clone(&self.signer),
                    rpc_client: Arc::clone(&self.rpc_client),
                    path: self.path.clone(),
                    error: self.error.clone(),
                    balance: self.balance.clone(),
                    receiver_value: self.receiver_value.clone(),
                    signature: self.signature.clone(),
                    is_loading: self.is_loading,
                    current_frame: self.current_frame,
                };
                Command::perform(transfer_sol(values), Message::TransactionExecuted)
            }
            Message::TransactionExecuted(Ok(signature)) => {
                self.signature = signature;
                let path = self
                    .path
                    .clone()
                    .unwrap_or_else(|| default_file().to_path_buf());
                self.is_loading = false;
                Command::perform(
                    display_balance(path, self.rpc_client.clone()),
                    Message::BalanceLoaded,
                )
            }
            Message::TransactionExecuted(Err(error)) => {
                self.error = Some(error);
                self.is_loading = false;
                Command::perform(async { time::sleep(Duration::from_secs(5)).await }, |_| {
                    Message::ErrorCleared
                })
            }
            Message::TxValuesHandler((address, amount)) => {
                self.receiver_value = (address, amount);
                Command::none()
            }
            Message::ErrorCleared => {
                self.error = None;
                Command::none()
            }
            Message::NextFrame => {
                self.current_frame = (self.current_frame + 1) % 21;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(75)).map(|_| Message::NextFrame)
    }

    fn view(&self) -> Element<'_, Message> {
        let image_path = match self.current_frame {
            0 => "./gif_animation/loader1.png",
            1 => "./gif_animation/loader2.png",
            2 => "./gif_animation/loader3.png",
            3 => "./gif_animation/loader4.png",
            4 => "./gif_animation/loader5.png",
            5 => "./gif_animation/loader6.png",
            6 => "./gif_animation/loader7.png",
            7 => "./gif_animation/loader7.png",
            8 => "./gif_animation/loader7.png",
            9 => "./gif_animation/loader7.png",
            10 => "./gif_animation/loader11.png",
            11 => "./gif_animation/loader12.png",
            12 => "./gif_animation/loader13.png",
            13 => "./gif_animation/loader14.png",
            14 => "./gif_animation/loader15.png",
            15 => "./gif_animation/loader16.png",
            16 => "./gif_animation/loader7.png",
            17 => "./gif_animation/loader7.png",
            18 => "./gif_animation/loader7.png",
            19 => "./gif_animation/loader7.png",
            20 => "./gif_animation/loader7.png",
            _ => "./gif_animation/loader1.png",
        };

        let balance_text = match self.balance {
            Some(balance) => column![
                text("SOL Balance: ").style(color!(0x30cbf2)).size(14),
                text(format!(" {:.3}", balance as f32 / LAMPORTS_PER_SOL as f32)).size(14)
            ],
            None => column![text("Loading balance...").size(14)],
        };

        let file_path = self
            .path
            .as_deref()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(""));

        let file_path_indicator = text("Path of your keypair:")
            .size(14)
            .style(color!(0x30cbf2));
        let file_path_name = text(file_path.to_str().unwrap_or(DEFAULT_LOCATION)).size(14);

        let display_path = column![file_path_indicator, file_path_name];

        let display_pkey = display_pubkey(file_path.to_path_buf());

        // display the pubkey of the keypair & SOL balance

        let wallet_info = row![display_pkey, balance_text].spacing(100);

        let load_keypair = button("Load keypair").on_press(Message::Open);

        // Solana sender

        let some_h2 = Column::new().push(Space::with_height(20)).push(
            text("Send SOL to any wallet!!! LFG")
                .style(color!(0x30cbf2))
                .size(14),
        );

        let address_input = text_input("Put receiver address", &self.receiver_value.0)
            .on_input(|value| Message::TxValuesHandler((value, self.receiver_value.1.to_string())));

        let amount_input = text_input("Lamports to send", &self.receiver_value.1.to_string())
            .on_input(|value| Message::TxValuesHandler((self.receiver_value.0.clone(), value)));

        let send_lamports_btn: Element<'_, Message> = if self.is_loading {
            Image::new(image_path).width(64).height(40).into()
        } else {
            button("Send lamports")
                .on_press(Message::ExecuteTransaction)
                .into()
        };

        let signature = text(&self.signature).size(14);

        // if there's some error, display it
        let info_message = if let Some(ref error) = &self.error {
            text(format!("Error: {:?}", error))
                .size(14)
                .style(color!(0xFF0000))
        } else {
            text("").size(1)
        };

        container(
            column![
                wallet_info,
                display_path,
                load_keypair,
                info_message,
                some_h2,
                address_input,
                amount_input,
                send_lamports_btn,
                signature
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
