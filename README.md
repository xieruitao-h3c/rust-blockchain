# rust-blockchain

A simple blockchain written in Rust. Calling it a *blockchain* is already a stretch; this is purely a playground for testing code and ideas.

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

**Fair warning:** This code uses concurrency so your CPU usage will spike. Avoid running for too long.

---

- Peers are identified by ports [4000-4004], as opposed to IP addresses with a single hardcoded port, that way it's easier to run on one machine (extending code to support IP addresses should be relatively straightforward).

- When you start a node in the `mine` mode, an unoccupied port will be auto-assigned and the process of mining will begin. In the `broadcast` mode, transactions are generated randomly and include a sender, receiver, amount [of coins] and a fee.

- The process of mining involves taking transactions from mempool (sorted by fees in descending order) and trying to find a hash that satisfies the difficulty condition by changing the nonce value. Once the block is mined, it's broadcast to the other nodes, and the process starts over on all the peers.

- The peers also support syncing mempool and blockchain. You can watch it in action by starting a second miner *late*. It will request a copy of the blockchain and mempool from others, and resume mining the same data.

- Besides one SHA256 function, there is zero cryptography in this repo.

## Contribute

PRs are welcome for existing code improvements, bug fixes and typos; however, extra features are not guaranteed to be merged in to keep this repo simple.

## Exercises

If you're inspired to build on top of this, here are some ideas to try on your own:

- Database support for the blockchain and mempool
- Support for uncle chains and reorgs
- A wallet implementation with real addresses/utxos
- All kinds of validation rules for blocks, transactions and nodes
- An implementation of some sort of discovery/gossip protocol
- An improvement on how nodes communicate

## License

MIT
