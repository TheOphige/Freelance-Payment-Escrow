Freelance Payment Escrow
(Placeholder banner; replace with a custom image showing escrow flow in a real repo.)
A decentralized escrow contract written in Rust for Optimism Stylus. This program provides a trust-minimized payment system for freelance work, where clients deposit funds that are held securely until work approval, with automatic release or refund mechanisms to resolve disputes efficiently.

🔑 Key Capabilities

Secure Deposits: Clients lock ETH for specific freelance jobs with a timeout deadline
Flexible Resolution: Manual release/refund by client or auto-release to freelancer after deadline
Dispute Minimization: Timeout-based auto-resolution to prevent indefinite holds
Job Management: Track job status, amounts, and parties involved
Ownership Controls: Admin-only functions for pausing and emergency interventions
Event Emission: Transparent logs for all escrow activity
Optimized Deployment: Lightweight and gas-conscious for Stylus on Optimism L2


Project Structure
The repository is organized for easy navigation and development:
Freelance-Payment-Escrow/
├── src/                        # Smart contract source code
│   └── lib.rs                  # Main escrow contract in Rust (Stylus)
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
│   └── integration_tests.rs    # Rust-based tests (deposit, release, etc.)
├── Cargo.toml                  # Rust dependencies and features
├── README.md                   # This file
└── LICENSE                     # MIT License


src/: Contains the core Rust contract logic (~150 lines).
frontend/: A simple React app with three main views (Client Dashboard, Freelancer Dashboard, Transaction History, ~300 lines).
tests/: Integration tests covering edge cases like invalid inputs and deadlines.
Total code: Lightweight, designed for a 24-hour prototype.


💡 Use Cases
The escrow can serve as the backbone for many freelance and payment applications:

Gig economy platforms like Upwork or Fiverr integrations
Remote work payment systems for developers and designers
Service-based marketplaces for consulting or creative services
Milestone-based project funding
Dispute resolution in peer-to-peer services
Multi-chain freelance hubs


🏗 Contract Design
The Escrow contract manages these main components:

Jobs: Each escrow entry tracks the client address, freelancer address, amount, deadline, and status (pending/released/refunded)
Timeout Logic: Automatic fund release to freelancer if client doesn't act by deadline
Admin Controls: Admin can pause operations, transfer ownership, and handle emergencies
State Tracking: Prevents double-spending, invalid releases, or refunds post-deadline


⚙️ Core Functions
Escrow Actions

initialize() → Deploy the contract (no constructor args needed)
deposit(freelancer: Address, duration: u64) → Client deposits ETH for a job (payable function)
release(job_id: u256) → Client releases funds to freelancer
refund(job_id: u256) → Client refunds before deadline
autoRelease(job_id: u256) → Freelancer claims funds after deadline

Administrative Functions

set_paused(state) → Pause/unpause escrow activity (admin only)
transfer_ownership(new_admin) → Transfer admin rights
emergency_refund(job_id: u256) → Force refund any job (admin only)

Read-Only Queries

get_job(job_id: u256) → Fetch details of a job
get_active_jobs() → Retrieve all pending job IDs
get_total_jobs() → Check total number of created jobs
is_paused() → View if the contract is paused


📢 Events
The contract emits structured logs for monitoring:

Deposited(job_id: u256, client: Address, freelancer: Address, amount: u256)
Released(job_id: u256, amount: u256)
Refunded(job_id: u256, amount: u256)
AutoReleased(job_id: u256, amount: u256)
PauseToggled(paused: bool)
OwnershipTransferred(old_admin: Address, new_admin: Address)


Quick Start
Prerequisites

Node.js (v18+)
Rust (for Stylus CLI)
MetaMask or similar wallet (configured for OP Sepolia)
Test ETH (from Superchain Faucet or QuickNode Optimism Faucet)

Installation Steps

Clone the repository:
git clone https://github.com/yourusername/Freelance-Payment-Escrow.git
cd Freelance-Payment-Escrow


Install front-end dependencies:
cd frontend
npm install


Install contract dependencies:
cd ..
cargo install --force cargo-stylus
rustup target add wasm32-unknown-unknown


Verify Stylus CLI:
cargo stylus --help


Compile the contract:
cargo stylus check


Generate Solidity-compatible ABI:
cargo stylus export-abi

The export-abi feature is enabled in Cargo.toml:
[features]
export-abi = ["stylus-sdk/export-abi"]


Deploy to OP Sepolia:
cargo stylus deploy \
    --endpoint https://sepolia.optimism.io \
    --private-key <yourprivatekey>


No constructor arguments are required.
Update frontend/src/App.js with the deployed contract address.



Deployed Address: 0x1234567890AbCdEf1234567890AbCdEf12345678 (Hypothetical; replace with actual address on OP Sepolia Explorer).

Start the front-end:cd frontend
npm start


App runs at http://localhost:3000.




Development & Testing
Expand Macros
To inspect the expanded Rust code from Stylus SDK macros:
cargo install cargo-expand
cargo expand --all-features --release --target=wasm32-unknown-unknown

Test Cases

Deposits: Create jobs with valid/invalid amounts and durations
Resolutions: Test release, refund, and auto-release scenarios
Timeouts: Simulate deadline expiry and claims
Admin Controls: Test pausing, ownership transfer, emergency refunds
Edge Cases: Invalid job IDs, post-deadline refunds, unauthorized calls
Events: Confirm all expected logs are emitted

Run tests:
cargo test


Usage
Smart Contract Interaction

Functions: Use the ABI (frontend/src/abi.json) for integration via ethers.js or similar.
Events: Query Deposited, Released, etc., for off-chain indexing and UI updates.
Explorer: Interact directly via OP Sepolia Explorer using the deployed address.

Web App Usage

Connect Wallet: Click "Connect Wallet" to link MetaMask (ensure OP Sepolia network).
Client Flow:
Navigate to Client Dashboard.
Enter freelancer address, amount (ETH), and duration (days).
Click "Deposit Funds" to create a job and lock funds.
View active jobs; click "Release Funds" to approve or "Request Refund" if unsatisfied (before deadline).


Freelancer Flow:
Navigate to Freelancer Dashboard.
View assigned jobs.
If deadline has passed and job is pending, click "Claim Funds".


Transaction History:
View all events (deposits, releases, refunds) with timestamps and transaction hashes (linked to OP Sepolia Explorer).



Example Scenario:

Client deposits 0.1 ETH for a 7-day job.
Freelancer completes work.
Client clicks "Release Funds" to transfer ETH.
If client ignores, freelancer claims funds after 7 days via "Claim Funds".

Local Testing:

Simulate on OP Sepolia using test ETH.
Use a local node or public RPC for development.


🔒 Security Considerations

Strict Payment Validation: Ensures exact amounts and prevents unauthorized access
Access Control: Only clients can release/refund; freelancers claim post-deadline
State Safety: Prevents double releases, refunds after deadline, or invalid operations
Timeout Protection: Automatic resolution to avoid fund locks
Emergency Tools: Admin can pause or force refunds
Input Validation: Checks addresses, timestamps, and amounts for correctness
Audits: This is a prototype; audit thoroughly before mainnet deployment to mitigate risks like reentrancy or overflows
Limitations: Lacks advanced dispute resolution (e.g., arbitration); extend with oracles for production use


Contributors

Your Name/Username: Lead Developer (Smart Contract & Front-End)
Grok (xAI): AI Assistant for ideation, code snippets, and UI design guidance
Open to contributions! Fork and submit PRs for improvements (e.g., multi-currency support, enhanced UI).


License
This project is licensed under the MIT License - see the LICENSE file for details.

Acknowledgments

Inspired by DeFi escrow patterns (e.g., OpenZeppelin contracts)
Built as a 24-hour project demo on September 23, 2025
Thanks to the Optimism and Stylus communities for documentation and tools

For issues or suggestions, open a GitHub issue. Happy freelancing! 🚀