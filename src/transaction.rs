use std::str::FromStr;

use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

use crate::{Error, SolExecApp};

fn parse_amount(amount_str: &str) -> Result<u64, Error> {
    let parts: Vec<&str> = amount_str.split('.').collect();

    let lamports = match parts.len() {
        // take only integer part
        1 => {
            let valid_integer = parts[0].parse::<u64>().unwrap_or(0);
            valid_integer
                .checked_mul(LAMPORTS_PER_SOL)
                .ok_or(Error::InvalidAmount)?
        }
        // take integer + decimal part
        2 => {
            let integer = parts[0].parse::<u64>().unwrap_or(0);

            // take decimal, example: 0.125, so decimal part is literal 125
            let fraction = &format!("{:0<9}", parts[1])[..9];
            let fraction_value = fraction.parse::<u64>().unwrap_or(0);

            // convert both parts in lamports and add them
            let whole_amount = integer
                .checked_mul(LAMPORTS_PER_SOL)
                .ok_or(Error::InvalidAmount)?;

            whole_amount
                .checked_add(fraction_value)
                .ok_or(Error::InvalidAmount)?
        }
        // if the input has more than one dot, an error throws
        _ => return Err(Error::InvalidAmount),
    };

    Ok(lamports)
}

pub async fn transfer_sol(values: SolExecApp) -> Result<String, Error> {
    let signer_pubkey = values.signer.pubkey();
    let to_address_str = &values.receiver_value.0;

    if to_address_str.as_bytes().len() < 32 {
        return Err(Error::InvalidPubKeyLen);
    }

    let to = Pubkey::from_str(&values.receiver_value.0).unwrap();
    let lamports = values.receiver_value.1;
    let amount_as_u64 = parse_amount(&lamports)?;

    if amount_as_u64 <= 0 {
        return Err(Error::InvalidAmount);
    } else if values.balance.unwrap_or(0) < amount_as_u64 {
        return Err(Error::InsufficientBalance);
    }

    let transfer_ix = system_instruction::transfer(&signer_pubkey, &to, amount_as_u64);
    let mut tx = Transaction::new_with_payer(&[transfer_ix], Some(&signer_pubkey));

    let send_cfg = RpcSendTransactionConfig {
        skip_preflight: true,
        preflight_commitment: Some(CommitmentLevel::Confirmed),
        encoding: Some(UiTransactionEncoding::Base64),
        max_retries: Some(3),
        min_context_slot: None,
    };

    let blockhash_result = values
        .rpc_client
        .get_latest_blockhash_with_commitment(values.rpc_client.commitment())
        .await;

    let blockhash = if let Ok((blockhash_info, _)) = blockhash_result {
        blockhash_info
    } else {
        return Err(Error::FetchBlockhashError);
    };

    tx.sign(&[&values.signer], blockhash);

    let signature_result = values
        .rpc_client
        .send_transaction_with_config(&tx, send_cfg)
        .await;

    let signature = if let Ok(signature_data) = signature_result {
        signature_data
    } else {
        return Err(Error::TransactionError);
    };

    loop {
        let commitment_config = CommitmentConfig::finalized();
        let confirmed = values
            .rpc_client
            .confirm_transaction_with_commitment(&signature, commitment_config)
            .await;
        let result = if let Ok(result) = confirmed {
            result
        } else {
            return Err(Error::TransactionError);
        };
        if result.value {
            break;
        }
    }

    Ok(signature.to_string())
}
