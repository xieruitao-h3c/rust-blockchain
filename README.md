# rust-blockchain

A simple blockchain written in Rust. Calling it a *blockchain* is a stretch; this is purely a playground for messing around with code and ideas.

## Get started

Requires [Rust](https://www.rust-lang.org/) to be installed locally.

- Run a node to start mining: `cargo run mine`
- Generate and broadcast transactions (in another window): `cargo run broadcast`
- Help: `cargo run -- --help`

## Design

This code has two main features:

1. Simulatation of transactions being generated and broadcast.
2. Nodes competing against each other to "mine" a new block.

The best example of this is to open two windows and run `cargo run mine` on both. And in the third window run `cargo run broadcast` -- you will see transactions being created and nodes occasionally generating new blocks.

---

- Since this is meant to be run on a single machine, peers are identified by ports [4000-4004], as opposed to IP addresses.

- When you start a node in the `mine` mode, an unoccupied port will be auto-assigned and the process of mining will begin. In the `broadcast` mode, transactions are generated randomly and include a sender, receiver, amount [of coins] and a fee.

- The process of mining involves taking transactions from mempool (sorted by fees in descending order) and trying to find a hash that satisfies the difficulty condition by changing the nonce value. Once the block is mined, it's broadcast to the other nodes, and the process starts over on all the peers.

- The peers also support syncing mempool and blockchain. You can watch it in action by starting a second miner *late*. It will request a copy of the blockchain and mempool from others, and resume mining the same data.

- There is hardly any cryptography here since this was not the main focus.

## Contribute

Idiomatic code improvements, bug fixes and general cleanup PRs are more than welcome. However, no promises on merging any major feature additions, since this repo is meant to stay light and simple.

## Exercises

If you'd like to build on top of this, here are some ideas you could try:

- Extend to support multiple machines
- Database integration to store transactions and blocks
- Support for uncle chains and reorgs
- A wallet implementation with addresses & utxos
- Better validation rules for blocks, transactions and nodes

## License

MIT
