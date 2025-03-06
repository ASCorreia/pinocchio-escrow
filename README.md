# Escrow Using Pinocchio

## Description
This project implements an escrow service using the Pinocchio framework. It allows users to create escrow transactions with three main instructions: Make, Take, and Refund.

## Instructions

### Make
The `Make` instruction is used to create a new escrow transaction. It requires the necessary parameters to set up the escrow.

### Take
The `Take` instruction allows the designated party to claim the assets held in escrow.

### Refund
The `Refund` instruction enables the original maker to reclaim the assets if the transaction does not proceed as planned.

## State Management
The state of the escrow transaction is managed using the `Escrow` struct, which includes:
- `maker`: The public key of the maker.
- `mint_x`: The mint address for the first asset.
- `mint_y`: The mint address for the second asset.
- `amount`: The amount of the asset being held in escrow.
- `bump`: A bump value for account derivation.

## Installation
To install the necessary dependencies, run:
```bash
cargo build
```

## Usage
To use the escrow program, deploy it on the Solana blockchain and interact with it using the provided instructions.

