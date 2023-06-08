use anyhow::Result;
use std::str::FromStr;

use strum::IntoEnumIterator;

use ethers::abi::Address;
use hyperlane_core::{HyperlaneDomainType, KnownHyperlaneDomain, H160};

pub fn show_domains() {
    println!("\n\nAvailable hyperlane domains:");
    for domain in KnownHyperlaneDomain::iter() {
        println!(
            "{}: [ID = {}]",
            domain.as_str().to_lowercase(),
            domain as u32
        );
    }
}

pub fn validate_domain(s: &str) -> Result<KnownHyperlaneDomain, String> {
    KnownHyperlaneDomain::from_str(s).map_err(|_| {
        show_domains();
        format!("{s} is an invalid domain")
    })
}

pub trait KnownHyperlaneDomainExtension {
    /// Obtains default paymaster contract address.
    fn paymaster_address(&self) -> Result<Address>;

    /// Obtains the origin hyperlane dispatch contract address and rpc url.
    fn dispatch_address_and_rpc_url(&self, override_url: Option<&String>)
        -> Result<(H160, String)>;
}

impl KnownHyperlaneDomainExtension for KnownHyperlaneDomain {
    fn paymaster_address(&self) -> Result<Address> {
        // https://docs.hyperlane.xyz/docs/resources/addresses
        let domain_type = self.domain_type();
        match domain_type {
            HyperlaneDomainType::Testnet => Ok(Address::from_str(
                "0xF90cB82a76492614D07B82a7658917f3aC811Ac1",
            )?),
            HyperlaneDomainType::Mainnet => Ok(Address::from_str(
                "0x56f52c0A1ddcD557285f7CBc782D3d83096CE1Cc",
            )?),
            _ => anyhow::bail!("Domain type is not yet supported: {domain_type}"),
        }
    }

    fn dispatch_address_and_rpc_url(
        &self,
        override_url: Option<&String>,
    ) -> Result<(H160, String)> {
        // https://docs.hyperlane.xyz/docs/resources/addresses
        let domain_type = self.domain_type();
        let origin_contract = match domain_type {
            HyperlaneDomainType::Testnet => {
                Address::from_str("0xCC737a94FecaeC165AbCf12dED095BB13F037685")?
            }
            HyperlaneDomainType::Mainnet => {
                Address::from_str("0x35231d4c2D8B8ADcB5617A638A0c4548684c7C70")?
            }
            _ => anyhow::bail!("TODO: Domain type {domain_type} is not yet supported"),
        };

        // TODO: drop hardcoded url and validate the passed url by querying
        // chain_id from the node.
        let url = match self {
            KnownHyperlaneDomain::Goerli => {
                "https://goerli.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161".to_string()
            }
            _ => anyhow::bail!("TODO: {} is not yet supported", self),
        };

        Ok((origin_contract, override_url.map_or(url, |u| u.to_string())))
    }
}
