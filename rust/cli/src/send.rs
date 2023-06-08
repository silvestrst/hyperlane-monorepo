use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use crate::abi::{GAS_PAYMASTER, MAILBOX, PAYMASTER_GAS};
use crate::domains::{validate_domain, KnownHyperlaneDomainExtension};
use crate::key;
use ethers::abi::Address;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use ethers::types::{Bytes, TransactionReceipt};
use ethers_providers::Middleware;
use hyperlane_core::{KnownHyperlaneDomain, H256};

use anyhow::Result;

#[derive(Debug, clap::Args)]
pub struct SendArgs {
    /// Original sender domain.
    #[arg(value_parser = validate_domain)]
    origin_domain: KnownHyperlaneDomain,
    /// Destination chain domain.
    #[arg(value_parser = validate_domain)]
    destination_domain: KnownHyperlaneDomain,
    /// Recipient contract on the destination chain.
    recipient: Address,
    /// Message to be sent to the recipient.
    message: String,
    /// Full path to the private key.
    private_key: PathBuf,
    /// URL override, useful when default CLI provided URL does not work.
    #[arg(long)]
    url: Option<String>,
}

impl SendArgs {
    pub async fn process(&self) -> Result<()> {
        let (origin_contract, url) = self
            .origin_domain
            .dispatch_address_and_rpc_url(self.url.as_ref())?;
        let (client, address) = self.get_client(&url)?;

        // Send the transaction to the destination chain. Please note, in order
        // for it to be processed by the relayers, the price has to be quoted
        // from the paymaster and paid.
        let mailbox = MAILBOX::new(origin_contract, client.clone());
        let dispatch_res = mailbox
            .dispatch(
                self.destination_domain as u32,
                H256::from(self.recipient).0,
                Bytes::from_str(&hex::encode(&self.message))?,
            )
            .send()
            .await?
            .log_msg("PENDING TRANSACTION HASH")
            .await?;
        println!("TRANSACTION SUBMITTED TO THE RELAYERS");

        let receipt =
            dispatch_res.ok_or_else(|| anyhow::anyhow!("Something went wrong! No TX receipt"))?;
        let message_id = self.get_message_id(receipt)?;
        println!("MESSAGE ID: {message_id:#x}");

        // TODO: possibly can be validated against dest address.
        let destination_domain = self.destination_domain as u32;
        let gas_paymaster =
            GAS_PAYMASTER::new(self.destination_domain.paymaster_address()?, client.clone());

        // The amount of gas that the recipient contract consumes on the destination
        // chain when handling a message from this origin chain.
        let quote = gas_paymaster
            .quote_gas_payment(destination_domain, PAYMASTER_GAS.into())
            .call()
            .await?;
        println!("GAS PAYMENT QUOTE: {quote}");

        // Pay for the transaction to be relayed to the recipient.
        gas_paymaster
            .pay_for_gas(
                message_id.into(),
                destination_domain,
                PAYMASTER_GAS.into(),
                address,
            )
            .value(quote)
            .send()
            .await?
            .log_msg("PENDING PAYMENT TRANSACTION HASH")
            .await?;
        println!("PAYMENT SUCCEEDED, TRANSACTION IS BEING RELAYED TO THE RECEPIENT");

        Ok(())
    }

    /// Creates an ethereum client with signing capabilities.
    fn get_client(&self, url: &str) -> Result<(Arc<impl Middleware>, Address)> {
        let wallet: LocalWallet = key::get_ethereum_signing_key(&self.private_key)?
            .parse::<LocalWallet>()?
            .with_chain_id(self.origin_domain as u32);
        let provider = Provider::<Http>::try_from(url)?;
        let middleware = SignerMiddleware::new(provider, wallet);
        let address = middleware.address();
        Ok((Arc::new(middleware), address))
    }

    /// Parses dispatch transaction receipt to obtain the HyperLane message id.
    fn get_message_id(&self, receipt: TransactionReceipt) -> Result<H256> {
        // Get the message ID from the transaction receipt logs.
        // Hardcoded hash is the `DispatchId` signature.
        let dispatch_id_hash =
            hex::decode("788dbc1b7152732178210e7f4d9d010ef016f9eafbe66786bd7169f56e0c353a")?;
        let dispatch_id_hash = H256::from_slice(dispatch_id_hash.as_slice());

        receipt
            .logs
            .iter()
            .find_map(|log| {
                if log.topics.first() == Some(&dispatch_id_hash) {
                    Some(log.topics[1])
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to obtain message ID"))
    }
}
