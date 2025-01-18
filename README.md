# NEAR pump

A robust smart contract implementation for creating and managing fungible tokens on the NEAR Protocol blockchain, with built-in auction functionality for initial token distribution.

## Overview

This project implements a token factory that allows users to create their own fungible tokens on NEAR blockchain with customizable parameters and an initial auction mechanism. The factory deploys individual token contracts with predefined standards and features.

## Features

* **Token Creation**:
  * Customizable token metadata
  * Configurable total supply
  * Built-in auction mechanism
  * Storage management
  * Standardized token interface

* **Token Configuration**:
  * Token name and symbol
  * Decimals specification
  * Custom icon and metadata
  * Reference data support
  * Description and image storage

* **Auction System**:
  * Configurable auction duration
  * Minimum buy amount setting
  * Order management
  * Automatic price calculation
  * Fair distribution mechanism

* **Storage Management**:
  * Efficient storage tracking
  * Deposit management
  * Cost calculation
  * Storage staking

## Prerequisites

* Rust (latest stable version)
* NEAR account (Testnet or Mainnet)
* NEAR CLI
* Environment setup for NEAR development

## Installation

1. Clone the repository:

```bash
git clone https://github.com/hasselalcala/near-pump.git
cd near-pump
```

2. Build the project:

```bash
cargo build
```

## Usage

### Creating a New Token

To create a new token, call the `create_token` function with the following parameters:

```rust
create_token(
spec, // Token specification (e.g., "ft-1.0.0")
name, // Token name
symbol, // Token symbol
icon, // Optional token icon
reference, // Optional reference data
reference_hash, // Optional reference hash
decimals, // Token decimals
image, // Token image
description, // Token description
auction_duration, // Duration of initial auction
min_buy_amount // Minimum buy amount for auction
)
```

### Storage Management

Users need to deposit storage fees before creating tokens.

## Contract Structure

The project consists of two main contracts:

1. **Token Factory Contract** (`token_factory/src/lib.rs`):
   * Manages token creation
   * Handles storage deposits
   * Tracks created tokens
   * Deploys individual token contracts

2. **Base Token Contract** (`base_token/src/lib.rs`):
   * Implements NEP-141 standard
   * Manages token transfers
   * Handles auction mechanism
   * Processes token distribution

## Events

The system emits standardized events for token creation:

```rust
RegisterTokenLog {
owner_id,
total_supply,
spec,
name,
symbol,
icon,
reference,
reference_hash,
decimals,
image,
description,
auction_duration,
min_buy_amount
}
```


## Architecture

The project follows a modular architecture with several key components:

* **Factory Module**: Manages token creation and deployment
* **Token Module**: Implements token functionality and standards
* **Events**: Handles event logging and notifications
* **Storage**: Manages contract storage and deposits
* **Auction**: Implements token distribution mechanism

## Security Features

* Storage staking requirements
* Deposit validation
* Owner-only functions
* Standard compliance checks
* Input validation

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

* NEAR Protocol team
* near-sdk-rs developers
* NEP-141 standard authors

