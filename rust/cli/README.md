# Simple CLI to interface on-chain Mailboxes

This CLI allows users to submit cross-chain messages. Please note that it is work in progress, so some features are limited. For example, at the moment it only supports sending messages from goerli testnet.

## Key features

- send command (user message dispatch from origin chain to the destination chain)
- query command (allows to query and filter messages originated on the specified original domain chain)
- key command (key generation and key related information)

## Must read guides

https://docs.hyperlane.xyz/docs/build-with-hyperlane/quickstarts/messaging
https://docs.hyperlane.xyz/docs/build-with-hyperlane/guides/manually-pay-for-interchain-gas
https://docs.hyperlane.xyz/docs/build-with-hyperlane/guides/paying-for-interchain-gas

## Quickstart and testing

1) Generate a new key using `cli key generate-ethereum`
2) Find out the address using `cli key show-ethereum-address`
3) Fund your address on goerli via one of the faucets. I use https://goerli-faucet.pk910.de/
4) Send a cross-chain transaction using `cli send`.
  - **Use goerli origin domain**
  - **You can use the TestRecipient contract for testing (0x36FdA966CfffF8a9Cdc814f546db0e6378bFef35)**

You can query the messages via `cli query` and also look up the messages directly on the hyperlane explorer:
https://explorer.hyperlane.xyz/

## TODO

Query:
- Pretty message data retrieval
- Message status

