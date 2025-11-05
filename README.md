# nervcores-app (Paxi Pioneers NFT Minting Project)

This is a full-stack NFT minting dApp (Decentralized Application) built for the Paxi Network. The project consists of two main components:
1.  **Smart Contract (Backend):** A custom CosmWasm contract (`paxi-pioneers`) written in Rust, handling all on-chain logic.
2.  **Web Application (Frontend):** A dApp (`paxi-mint-frontend`) built with React/Vite, allowing users to interact with the smart contract.

---

## 1. Backend: Smart Contract (`paxi-pioneers`)

The `paxi-pioneers` directory (originally from `src`, `Cargo.toml`, etc. in the root) contains the custom CosmWasm smart contract.

### Core Contract Features:
* **Collection:** Paxi Pioneers (Symbol: PIONEER)
* **Supply:** 10,000 NFTs
* **Price:** 10 PAXI (`10000000upaxi`) per NFT
* **Public Minting:** Supports `PublicMint` (1 NFT) and `PublicBatchMint` (up to 10 NFTs per transaction).
* **Royalties:** Implements the CW2981 standard (7.5% for the admin).
* **Admin Controls:** A full admin panel for:
    * `Withdraw`: Withdraw funds from the contract.
    * `PauseMint` / `UnpauseMint`: Pause or resume the public mint.
    * `UpdateBaseUri`: To manage the "reveal" process.
    * `SetProvenanceHash`: To set the collection's provenance hash for transparency.
* **Security:** The contract itself (`env.contract.address`) is set as the `minter` to allow for a secure public minting function.

### How to Build & Optimize the Backend (from a Linux/Ubuntu Environment)

This process is only necessary if you modify the smart contract logic.

1.  **Set up Rust Environment:**
    Ensure you have `rustup`, `cargo`, and the `wasm32-unknown-unknown` target.
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
    source "$HOME/.cargo/env"
    rustup target add wasm32-unknown-unknown
    ```

2.  **Set up Docker:**
    Ensure Docker is installed and running.

3.  **Compile & Optimize:**
    This command compiles the Rust code, optimizes it, and produces a final `.wasm` file that is compatible with Paxi (with the `-bulk-memory` feature disabled).

    ```bash
    # Navigate to the backend directory (e.g., paxi-pioneers)
    cd paxi-pioneers 
    
    # Run the optimizer
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      -e RUSTFLAGS='-C target-feature=-bulk-memory' \
      cosmwasm/rust-optimizer:0.17.0
    ```

4.  **Result:**
    The final `.wasm` file will be available in the `paxi-pioneers/artifacts/` directory.

---

## 2. Frontend: Web Application (`paxi-mint-frontend`)

The `paxi-mint-frontend` directory contains the React/Vite dApp for interacting with the smart contract.

### Core Frontend Features:
* "Galactic Night" dark-mode design.
* Smart wallet connection (detects `Keplr`, `Leap`, and `PaxiHub`).
* Live supply progress bar.
* Quantity selector (+/-) for minting 1-10 NFTs.
* Dynamic mint button that displays the total price.
* Admin-only panel (visible to admin) for `Withdraw`, `Pause`, `Reveal`, etc.
* "Your Collection" NFT gallery that fetches data on-chain.
* "Toast" notification system for transaction status.

### How to Run the Frontend (Local Development)

These are the steps to run the website on your local machine for testing.

1.  **Navigate to the Frontend Directory:**
    Open your terminal and navigate to the frontend folder.
    ```bash
    # Adjust this path based on your folder location
    cd /path/to/nervcores-app/paxi-mint-frontend 
    ```

2.  **Install Dependencies (One-Time Setup):**
    This command downloads all the required libraries for the project (React, CosmJS, etc.).
    ```bash
    npm install
    ```

3.  **Run the Development Server:**
    This command starts the local web server (usually at `http://localhost:5174/`) and will auto-refresh the page as you make code changes.
    ```bash
    npm run dev
    ```

---

## 3. Key Network Information (Paxi Testnet)

* **Contract Address:** `paxi1rnraxu54huv7wkmpff3v8mzhqh49g0e60x4ehzws4ee6upcy78sqc55vx0`
* **RPC Endpoint:** `https://testnet-rpc.paxinet.io`
* **LCD Endpoint:** `https://testnet-lcd.paxinet.io`
* **Chain ID:** `paxi-testnet`
