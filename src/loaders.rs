use std::{path::PathBuf, sync::Arc};

use crate::{errors::Error, Message};
use iced::{
    color, widget::{column, text}, Element
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::{read_keypair_file, Keypair},
    signer::Signer,
};

pub fn display_pubkey(file_path: PathBuf) -> Element<'static, Message> {
    let keypair = load_keypair_from_file(file_path);

    let label = text(format!("Wallet address: ",))
        .size(14)
        .style(color!(0x30cbf2));

    let value = text(keypair.pubkey().to_string()).size(14);

    let pubkey_container = column![label, value];
    pubkey_container.into()
}

pub fn load_keypair_from_file(path: PathBuf) -> Keypair {
    let keypair = read_keypair_file(path).unwrap_or(Keypair::new());
    keypair
}

pub async fn display_balance(path: PathBuf, rpc_client: Arc<RpcClient>) -> Result<u64, Error> {
    let keypair = load_keypair_from_file(path);
    rpc_client
        .get_balance(&keypair.pubkey())
        .await
        .map_err(|_| Error::FetchBalanceError)
}
