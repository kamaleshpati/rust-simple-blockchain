# ElemChain

[![tests](https://github.com/AlexKaravaev/elemchain/actions/workflows/tests.yml/badge.svg)](https://github.com/AlexKaravaev/elemchain/actions/workflows/tests.yml)

![Alt Text](./media/demo.gif)

Elementary and simple cryptocurrency implemented in Rust.

I started creating it just as simple blockchain example, but ended up in half way of creating own kind of a cryptocurrency.


Mostly this project is done to get familiar with Rust language and broaden understanding of how cryptocurrency/blockchain systems do work. So this repo is not even close to such distrtibuted systems as eth, bitcoin etc.

## What it can and can't do

What it can do:
* Generate transactions.
* Mine blocks from transactions.
* Broadcast new created blocks to the network and check validity of synchronized chains.
* Generate hashes of the block and check validity of the blocks, so that blockchain becomes immutable.

Things to be done in the future:

* Merkle roots instead of just having all txs in the block.
* No mempool with pending transactions. For now they are only displayed for local node(if this node has done it, then only this node can mine it)
* No wallet logic at all. Transaction are always 100 amount of coins send to some random peer in the network. So it's not even checked whether sender have this amount in hands.
* Now all of the blockchain is broadcasted to the network on each user interaction with cli app. To be honest, I just don't know how this part in cryptocurrency works. I guess we should only send blocks, when they are mined, but then when do we get the chain from other peers? Only on init?
* and many many other things


## Usage

Launch ```cargo run``` and then you will see a cli menu. It's kind of a playground. You can generate transacations, view other p2p nodes, view transactions that were not yet confirmed by miners, also you can mine pending txs too.

It's better to launch 2-3 nodes in separate terminals via ```cargo run``` too to see how they will reach consensus.

## Credits

I've found this code and websites useful:
* [Simple proof-of-work blockchain written in Rust](https://github.com/thor314/rust-blockchain)
* Make your own cryptocurrency from scratch
  * [Code](https://github.com/nathan-149/CustomCryptocurrency)
  * [Video](https://www.youtube.com/watch?v=malwhCwEosk)
* [How to build a blockchain in Rust](https://blog.logrocket.com/how-to-build-a-blockchain-in-rust/)
