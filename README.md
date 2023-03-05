<div align="center">
<h1 align="center">SplitMate - Smart Contract</h1>

[![Built with ink!](https://raw.githubusercontent.com/paritytech/ink/master/.images/badge_flat.svg)](https://github.com/paritytech/ink)
![Twitter Follow](https://img.shields.io/twitter/follow/NeoPowerDigital?style=social)

</div>

<div align="center">
<img src="https://user-images.githubusercontent.com/107150702/222936409-cc129632-3e78-46b1-ae6a-466a3d862f67.png" width="40%">
</div>

<div align="center">
<b>SplitMate</b> is an app to simplify the way we <b>share expenses</b> within a group of friends. 
It allows users to <b>track and split expenses</b> such as concert tickets, vacations, dinners, and other shared bills. <b>With Splitmate, users can avoid misunderstandings.</b>
</div>

# Docs

## UI

The frontend code repository is **[here](https://github.com/NeoPower-Digital/splitmate-ui)** 

## Environment setup

The smart contract is built using [ink!](https://use.ink/), an eDSL to write smart contracts in Rust for Polkadot blockchains built on the [Substrate](https://github.com/paritytech/substrate) framework.  

To compile the smart contract, Rust and Cargo are required. Here is an [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

[cargo-contract](https://github.com/paritytech/cargo-contract) is required too. Install it using this command:

```shell
cargo install cargo-contract --force --locked
```

## Build smart contract

To build the smart contract and generates the optimized WebAssembly bytecode, the metadata and bundles, execute this command:

```shell
cargo contract build --release
```

## Run tests off-chain

To run the tests off-chain, execute this command:

```shell
cargo test
```

## Upload & instantiate

Open the [Substrate Contracts-UI](https://contracts-ui.substrate.io).

Choose a chain (E.g. `Contracts (Rococo)` or `Shibuya`) in the dropdown placed in the top section of the left menu.

Follow the [official ink! guide](https://use.ink/getting-started/deploy-your-contract/#using-the-contracts-ui) to upload and instantiate the smart contract.

## ink! version

`ink`: 4.0.0

https://github.com/paritytech/ink/tree/v4.0.0