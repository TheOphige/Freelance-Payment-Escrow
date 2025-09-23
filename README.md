# Freelance-Payment-Escrow

![Freelance Escrow Banner](https://via.placeholder.com/1200x400.png?text=Freelance+Payment+Escrow+Project)  
*(Placeholder banner; replace with a custom image showing escrow flow in a real repo.)*

## Overview

The **Freelance Payment Escrow** is a decentralized application (DApp) built using Stylus smart contracts on the Optimism (OP) Sepolia testnet. It addresses the real-world problem of unreliable payments in freelance work, where approximately 20% of freelancers face non-payment issues (based on industry reports from platforms like Upwork). This project provides a trustless escrow system that holds client funds until the work is approved, reducing disputes and building confidence in gig economies.

Key features:
- **Secure Fund Holding**: Funds are locked in the smart contract until release or refund conditions are met.
- **Timeout-Based Resolution**: Automatic release to freelancer after a deadline if the client doesn't act.
- **Simple UI**: Web-based dashboards for clients and freelancers to interact with the contract.
- **Gas Efficiency**: Written in Stylus (Rust-based for EVM compatibility), ensuring low-cost transactions on Layer 2.

This project is designed as a 24-hour prototype, making it ideal for hackathons or quick MVPs. It demonstrates core blockchain concepts like escrow logic, events, and front-end integration.

## Tech Stack
- **Smart Contract**: Stylus (Rust-like syntax for Optimism EVM).
- **Blockchain**: Optimism Sepolia Testnet (L2 for Ethereum).
- **Front-End**: React.js with Tailwind CSS and ethers.js for wallet/contract interaction.
- **Tools**: Stylus CLI for compilation, Foundry/Hardhat for testing and deployment, MetaMask for wallet.
- **Other**: OP Sepolia faucet for test ETH.

## Project Structure

The repository is organized for easy navigation and development:

```
Freelance-Payment-Escrow/
├── contracts/                  # Smart contract source code
│   └── Escrow.stylus           # Main escrow contract in Stylus (Rust syntax)
├── frontend/                   # React-based front-end application
│   ├── src/
│   │   ├── components/         # Reusable UI components (e.g., JobCard, WalletConnect)
│   │   │   ├── ClientDashboard.js
│   │   │   ├── FreelancerDashboard.js
│   │   │   └── TransactionHistory.js
│   │   ├── App.js              # Main app entry point
│   │   ├── abi.json            # Compiled contract ABI
│   │   └── index.js            # React root
│   ├── public/                 # Static assets (e.g., index.html, favicon)
│   └── package.json            # Dependencies (react, ethers, tailwindcss)
├── tests/                      # Unit tests for the contract
│   └── Escrow.test.js          # Foundry/Hardhat tests (deposit, release, etc.)
├── scripts/                    # Deployment scripts
│   └── deploy.js               # Hardhat script to deploy to OP Sepolia
├── .gitignore                  # Git ignore file
├── hardhat.config.js           # Hardhat configuration for OP Sepolia
├── README.md                   # This file
└── LICENSE                     # MIT License
```

- **contracts/**: Contains the core Stylus contract logic.
- **frontend/**: A simple React app with three main views (Client Dashboard, Freelancer Dashboard, Transaction History).
- **tests/**: Basic tests covering edge cases like deadline expiry and invalid actions.
- Total code: ~200 lines (contract) + ~300 lines (front-end), keeping it lightweight.

## Installation

### Prerequisites
- Node.js (v18+)
- Rust (for Stylus CLI)
- MetaMask or similar wallet (configured for OP Sepolia)
- Test ETH (from [Optimism Faucet](https://app.optimism.io/faucet))

### Steps
1. Clone the repository:
   ```
   git clone https://github.com/yourusername/Freelance-Payment-Escrow.git
   cd Freelance-Payment-Escrow
   ```

2. Install front-end dependencies:
   ```
   cd frontend
   npm install
   ```

3. Set up Stylus and Hardhat:
   - Install Stylus CLI: Follow [Optimism Docs](https://docs.optimism.io/builders/app-developers/contracts/stylus).
   - Install Hardhat: `npm install --save-dev hardhat @nomiclabs/hardhat-ethers ethers`
   - Configure `hardhat.config.js` with your OP Sepolia RPC URL and private key.

4. Compile the contract:
   ```
   stylus compile contracts/Escrow.stylus
   ```

## Deployment

1. Deploy the smart contract to OP Sepolia:
   ```
   npx hardhat run scripts/deploy.js --network opSepolia
   ```
   - This will output the deployed contract address. Update `frontend/src/App.js` with this address and ABI.

2. Start the front-end locally:
   ```
   cd frontend
   npm start
   ```
   - App runs at `http://localhost:3000`.

## Usage

### Smart Contract Interaction (via Code or Etherscan)
- **Deployed Address**: `0x1234567890AbCdEf1234567890AbCdEf12345678` (Hypothetical; replace with your actual deployment on OP Sepolia Explorer: [Link](https://sepolia-optimism.etherscan.io/address/0x1234567890AbCdEf1234567890AbCdEf12345678)).
- **Functions**:
  - `deposit(freelancer: Address, duration: u64)`: Client deposits ETH for a job (payable).
  - `release(job_id: u256)`: Client releases funds to freelancer.
  - `refund(job_id: u256)`: Client refunds before deadline.
  - `autoRelease(job_id: u256)`: Freelancer claims after deadline.
- **Events**: `Deposited`, `Released`, `Refunded` for tracking.

### Web App Usage
1. **Connect Wallet**: Open the app, click "Connect Wallet" (ensures OP Sepolia network).
2. **Client Flow**:
   - Go to Client Dashboard.
   - Enter freelancer address, amount (ETH), duration (days).
   - Click "Deposit Funds" → Job created and funds locked.
   - View active jobs; click "Release Funds" on approval or "Request Refund" if unsatisfied (before deadline).
3. **Freelancer Flow**:
   - Go to Freelancer Dashboard.
   - View assigned jobs.
   - If deadline passed and pending, click "Claim Funds".
4. **Transaction History**:
   - View all events (deposits, releases, refunds) with timestamps and tx hashes (links to explorer).

**Example Scenario**:
- Client deposits 0.1 ETH for a 7-day job.
- Freelancer completes work.
- Client releases → Funds transferred.
- If client ignores, freelancer claims after 7 days.

**Testing**:
- Run tests: `npx hardhat test`.
- Local simulation: Use Hardhat fork of OP Sepolia.

## Security Considerations
- **Audits**: This is a prototype; audit before mainnet use (potential reentrancy or overflow risks).
- **Best Practices**: Uses Rust's type safety; restrict functions to authorized callers.
- **Limitations**: No advanced disputes (e.g., arbitration); extend with oracles if needed.

## Contributors
- **Your Name/Username**: Lead Developer (Smart Contract & Front-End).
- **Grok (xAI)**: AI Assistant for ideation, code snippets, and UI design guidance.
- Open to contributions! Fork and PR for improvements (e.g., multi-currency support).

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments
- Inspired by DeFi escrow patterns (e.g., OpenZeppelin examples).
- Built as a 24-hour project demo on September 23, 2025.

For issues or suggestions, open a GitHub issue. Happy freelancing! 🚀