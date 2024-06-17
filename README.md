## Casper Odra tutorials

Welcome to the Casper Association's Odra tutorials area. The team at Casper Developer Relations have gathered here a number of applications of the Odra framework to build smart contracts on the Casper platform. These should help you to get started with building your own smart contracts on Casper.  

### Counter  
A simple "counter" smart contract using Odra. This is a relatively simple contract, with the idea being that you can use this as your gateway into the world of Odra programming. We cover the approach to building this contract both in Casper 1.x and using Odra, in order to demonstrate the differences for developers coming from the Casper 1.x environment.  
[To the tutorial](./counter/tutorial.md)

### Donation 
In this tutorial, you will learn how to create a donation contract using Odra. This smart contract can accept funds from anyone, and funds can be withdrawn by the original deployer. The donation contract will introduce two new concepts in Odra development, not covered in the previous tutorials:
- payable entrypoints
- emitting events

[To the tutorial](./donation/tutorial.md)

### Election
This tutorial will guide you through the creation of a voting smart contract using Odra. The contract will be built assuming the following principles:
The deployer can specify candidates and a final voting time in the constructor.
The final voting time is denominated in block height.
The deployer cannot modify the candidates or end time after deployment.
Any account, besides the deployer, may make one vote for any candidate they please.
The contract could be extended to allow for modifications to candidates, end time, and voting capabilities, but this tutorial avoids these functionalities in the interest of simplicity.  
[To the tutorial](./election/tutorial.md)

### Escrow 
Escrow contracts are common and useful agreements for arbitrating arrangements between two or more parties. This tutorial will teach you how to create a basic escrow smart contract between two accounts with a dedicated arbiter.  
[To the tutorial](./escrow/tutorial.md)

### Odra x Fondant
Bridging the Gap for Casper Smart Contract Development & Testing
Odra is the recommended framework for building smart contracts on the Casper Network. Fondant, a new and exciting tool, simplifies running a local Casper network and testing contracts with its intuitive UI. As both tools evolve, we can expect closer integration in the future.
This guide will demonstrate how to combine Odra and Fondant today. We'll create a simple Odra contract, deploy it and test it on a local network using livenet. We'll also provide a script to fetch secret keys from Fondant for seamless interaction.  
[To the tutorial](./fondant_x_odra/tutorial.md)

### Recoverable Wallet
This tutorial creates a smart contract that behaves like a personal wallet with some additional features on top, demonstrating the concept of account abstraction. Some features enabled by this concept include:
 - Social recovery using trusted addresses to recover the account in case you lost it
 - Daily transaction limits
 - Allow lists for transfers exceding a given amount of tokens

In this example we implement the social recovery feature. Where a user can set a list of trusted addresses that in case of a lost key to this wallet can recover the funds and transfer them to a new account.  
[To the tutorial](./recoverable_wallet/tutorial.md)

### Zero to Hero with NFTs: Part 1
A simple NFT contract on the Casper testnet using Odra.  
[To the tutorial](./nft_zero_to_hero/part1/tutorial.md)

Zero to Hero with NFTs: Part 2 - Batch Minting with Nested CEP-78 Module
Enhanced NFT contract with batch minting.  
[To the tutorial](./nft_zero_to_hero/part2/tutorial.md)

---
### What is Odra?
Odra is the next-gen smart contract development framework for the Casper blockchain. 

### How can I get Odra?
[Install Odra](https://odra.dev/docs/getting-started/installation/)

### More information on Odra
Github page  
https://github.com/odradev/odra

Documentation  
https://odra.dev/docs/

---
Questions?

 - mail us at [devrel@casper.network](mailto:devrel@casper.network)
 - contact us on our Telegram channel https://t.me/csprdr