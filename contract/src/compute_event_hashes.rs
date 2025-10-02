use alloy_primitives::{keccak256, B256};

fn main() {
    // List of event signatures from the Escrow contract
    let event_signatures = vec![
        "Deposited(uint256,address,address,uint256)",
        "Released(uint256,uint256)",
        "Refunded(uint256,uint256)",
        "AutoReleased(uint256,uint256)",
        "EmergencyRefunded(uint256,address)",
        "PauseToggled(bool)",
        "OwnershipTransferred(address,address)",
    ];

    // Compute and print Keccak-256 hashes for each event
    for signature in event_signatures {
        // Compute Keccak-256 hash of the event signature
        let hash: B256 = keccak256(signature.as_bytes());
        // Format as hex string with 0x prefix
        println!("Event: {}", signature);
        println!("Keccak-256 Hash: 0x{}", hex::encode(hash));
        println!();
    }
}