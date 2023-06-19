use std::sync::Arc;

use crate::abi::MAILBOX;
use crate::domains::{validate_domain, KnownHyperlaneDomainExtension};
use ethers::abi::Address;
use ethers::providers::{Http, Provider};
use ethers_providers::Middleware;
use hyperlane_core::{KnownHyperlaneDomain, H256};

use anyhow::Result;

#[derive(Debug, clap::Args)]
pub struct QueryArgs {
    /// Hyperlane contract domain to query messages from.
    #[arg(value_parser = validate_domain)]
    origin_domain: KnownHyperlaneDomain,
    /// Block to query historic data from. Queries last 1000 blocks by default.
    #[arg(long)]
    from_block: Option<u64>,
    /// Original sender of the transaction.
    #[arg(long)]
    sender_address: Option<Address>,
    /// Hyperlane destination chain domain.
    #[arg(value_parser = validate_domain, long)]
    destination_domain: Option<KnownHyperlaneDomain>,
    #[arg(long)]
    /// Recipient contract on the destination domain.
    recipient: Option<Address>,
}

impl QueryArgs {
    pub async fn process(&self) -> Result<()> {
        let (contract, url) = self.origin_domain.dispatch_address_and_rpc_url(None)?;

        let client = Arc::new(Provider::<Http>::try_from(url)?);
        let mailbox = MAILBOX::new(contract, client.clone());

        let mut dispatch_filter = mailbox.dispatch_filter();
        let mut dispatch_id_filter = mailbox.dispatch_id_filter();

        if let Some(block) = self.from_block {
            dispatch_filter = dispatch_filter.from_block(block);
            dispatch_id_filter = dispatch_id_filter.from_block(block);
        } else {
            let latest_block = client.get_block_number().await?;
            dispatch_filter = dispatch_filter.from_block(latest_block - 1000);
            dispatch_id_filter = dispatch_id_filter.from_block(latest_block - 1000);
        }

        let dispatch_logs = dispatch_filter.query().await?;
        let dispatch_id_logs = dispatch_id_filter.query().await?;

        if dispatch_logs.len() != dispatch_id_logs.len() {
            anyhow::bail!("Dispatch log count != dispatch id logs count");
        }

        // Bind two event type logs together for filtering.
        let logs = dispatch_logs
            .iter()
            .zip(dispatch_id_logs.iter())
            .filter(|(dispatch, _)| {
                if self.sender_address.is_some() {
                    Some(dispatch.sender) == self.sender_address
                } else if self.destination_domain.is_some() {
                    Some(dispatch.destination)
                        == self.destination_domain.map(|domain| domain as u32)
                } else if self.recipient.is_some() {
                    Some(dispatch.recipient) == self.recipient.map(|address| H256::from(address).0)
                } else {
                    true
                }
            });

        for (dispatch, dispatch_id) in logs {
            println!("\nContract address: {:x}", contract);
            println!("Sender: {:x}", dispatch.sender);
            println!("Destination: {}", dispatch.destination);
            println!("Recipient: {:x}", H256::from(dispatch.recipient));
            println!("Hyperlane Message ID: {}", dispatch_id);
        }

        Ok(())
    }
}
