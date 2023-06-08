use ethers::prelude::abigen;

abigen!(
    MAILBOX,
    "../chains/hyperlane-ethereum/abis/Mailbox.abi.json"
);

abigen!(
    GAS_PAYMASTER,
    "../chains/hyperlane-ethereum/abis/IInterchainGasPaymaster.abi.json"
);

/// Fixed paymaster payment transaction gas.
///
/// Value has been taken from:
/// https://docs.hyperlane.xyz/docs/build-with-hyperlane/guides/paying-for-interchain-gas
pub const PAYMASTER_GAS: u32 = 100000;
