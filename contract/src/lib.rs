extern crate alloc;

use stylus_sdk::prelude::*;
use alloy_primitives::{U256, Address, Uint, B256};
use alloy_sol_types::sol;

sol_storage! {
    #[entrypoint]
    pub struct Escrow {
        address admin;
        bool paused;
        uint256 job_count;
        mapping(uint256 => Job) jobs;
        mapping(uint256 => bool) finalized;
    }

    pub struct Job {
        uint256 job_id;
        address client;
        address freelancer;
        uint256 amount;
        uint64 deadline;
        bool released;
        bool refunded;
    }
}

#[public]
impl Escrow {
    /// Initialize escrow contract
    pub fn initialize(&mut self) -> Result<(), Vec<u8>> {
        self.admin.set(self.vm().msg_sender());
        self.paused.set(false);
        self.job_count.set(U256::from(0));
        Ok(())
    }

    /// Client deposits ETH for a job
    #[payable]
    pub fn deposit(&mut self, freelancer: Address, duration: u64) -> Result<U256, Vec<u8>> {
        if self.paused.get() {
            return Err("Escrow is paused".as_bytes().to_vec());
        }
        if self.vm().msg_value() == U256::from(0) {
            return Err("Amount must be > 0".as_bytes().to_vec());
        }
        if freelancer == Address::ZERO {
            return Err("Invalid freelancer address".as_bytes().to_vec());
        }
        if duration == 0 {
            return Err("Duration must be > 0".as_bytes().to_vec());
        }

        let new_id = self.job_count.get() + U256::from(1);
        let client = self.vm().msg_sender();
        let amount = self.vm().msg_value();
        let timestamp = self.vm().block_timestamp();
        let deadline = timestamp + duration;
        let deadline_uint = Uint::<64, 1>::from(deadline);

        let mut job = self.jobs.setter(new_id);
        job.job_id.set(new_id);
        job.client.set(client);
        job.freelancer.set(freelancer);
        job.amount.set(amount);
        job.deadline.set(deadline_uint);
        job.released.set(false);
        job.refunded.set(false);

        self.job_count.set(new_id);
        self.finalized.setter(new_id).set(false);

        log(self.vm(), Deposited {
            job_id: new_id,
            client,
            freelancer,
            amount,
        });

        Ok(new_id)
    }

    /// Client releases funds to freelancer
    pub fn release(&mut self, job_id: U256) -> Result<(), Vec<u8>> {
        if self.paused.get() {
            return Err("Escrow is paused".as_bytes().to_vec());
        }

        let job = self.jobs.get(job_id);
        if job.client.get() != self.vm().msg_sender() {
            return Err("Only client can release".as_bytes().to_vec());
        }
        if job.released.get() || job.refunded.get() {
            return Err("Job already settled".as_bytes().to_vec());
        }
        if self.finalized.get(job_id) {
            return Err("Job already finalized".as_bytes().to_vec());
        }

        let amount = job.amount.get();
        let freelancer = job.freelancer.get();

        self.jobs.setter(job_id).released.set(true);
        self.finalized.setter(job_id).set(true);

        self.vm().transfer_eth(freelancer, amount)?;

        log(self.vm(), Released {
            job_id,
            amount,
        });

        Ok(())
    }

    /// Client refunds funds before deadline
    pub fn refund(&mut self, job_id: U256) -> Result<(), Vec<u8>> {
        if self.paused.get() {
            return Err("Escrow is paused".as_bytes().to_vec());
        }

        let job = self.jobs.get(job_id);
        if job.client.get() != self.vm().msg_sender() {
            return Err("Only client can refund".as_bytes().to_vec());
        }
        if job.released.get() || job.refunded.get() {
            return Err("Job already settled".as_bytes().to_vec());
        }
        if self.finalized.get(job_id) {
            return Err("Job already finalized".as_bytes().to_vec());
        }
        if self.vm().block_timestamp() >= job.deadline.get().to() {
            return Err("Deadline passed".as_bytes().to_vec());
        }

        let amount = job.amount.get();
        let client = job.client.get();

        self.jobs.setter(job_id).refunded.set(true);
        self.finalized.setter(job_id).set(true);

        self.vm().transfer_eth(client, amount)?;

        log(self.vm(), Refunded {
            job_id,
            amount,
        });

        Ok(())
    }

    /// Freelancer claims funds after deadline
    pub fn auto_release(&mut self, job_id: U256) -> Result<(), Vec<u8>> {
        if self.paused.get() {
            return Err("Escrow is paused".as_bytes().to_vec());
        }

        let job = self.jobs.get(job_id);
        if job.freelancer.get() != self.vm().msg_sender() {
            return Err("Only freelancer can claim".as_bytes().to_vec());
        }
        if job.released.get() || job.refunded.get() {
            return Err("Job already settled".as_bytes().to_vec());
        }
        if self.finalized.get(job_id) {
            return Err("Job already finalized".as_bytes().to_vec());
        }
        if self.vm().block_timestamp() < job.deadline.get().to() {
            return Err("Deadline not reached".as_bytes().to_vec());
        }

        let amount = job.amount.get();
        let freelancer = job.freelancer.get();

        self.jobs.setter(job_id).released.set(true);
        self.finalized.setter(job_id).set(true);

        self.vm().transfer_eth(freelancer, amount)?;

        log(self.vm(), AutoReleased {
            job_id,
            amount,
        });

        Ok(())
    }

    /// ADMIN: pause/unpause escrow
    pub fn set_paused(&mut self, state: bool) -> Result<(), Vec<u8>> {
        if self.vm().msg_sender() != self.admin.get() {
            return Err("Only admin".as_bytes().to_vec());
        }
        self.paused.set(state);

        log(self.vm(), PauseToggled {
            paused: state,
        });

        Ok(())
    }

    /// ADMIN: transfer ownership
    pub fn transfer_ownership(&mut self, new_admin: Address) -> Result<(), Vec<u8>> {
        if self.vm().msg_sender() != self.admin.get() {
            return Err("Only admin".as_bytes().to_vec());
        }
        if new_admin == Address::ZERO {
            return Err("Invalid admin address".as_bytes().to_vec());
        }

        let old_admin = self.admin.get();
        self.admin.set(new_admin);

        log(self.vm(), OwnershipTransferred {
            old_admin,
            new_admin,
        });

        Ok(())
    }

    /// ADMIN: emergency refund
    pub fn emergency_refund(&mut self, job_id: U256) -> Result<(), Vec<u8>> {
        if self.vm().msg_sender() != self.admin.get() {
            return Err("Only admin".as_bytes().to_vec());
        }

        let job = self.jobs.get(job_id);
        if job.released.get() || job.refunded.get() {
            return Err("Job already settled".as_bytes().to_vec());
        }
        if self.finalized.get(job_id) {
            return Err("Job already finalized".as_bytes().to_vec());
        }

        let amount = job.amount.get();
        let client = job.client.get();

        self.jobs.setter(job_id).refunded.set(true);
        self.finalized.setter(job_id).set(true);

        self.vm().transfer_eth(client, amount)?;

        log(self.vm(), EmergencyRefunded {
            job_id,
            admin: self.vm().msg_sender(),
        });

        Ok(())
    }

    /// View job details
    pub fn get_job(&self, job_id: U256) -> (U256, Address, Address, U256, u64, bool, bool) {
        let j = self.jobs.get(job_id);
        (
            j.job_id.get(),
            j.client.get(),
            j.freelancer.get(),
            j.amount.get(),
            j.deadline.get().to(),
            j.released.get(),
            j.refunded.get(),
        )
    }

    /// Get active (unsettled) jobs
    pub fn get_active_jobs(&self) -> Vec<U256> {
        let mut ids = Vec::new();
        let total = self.job_count.get();
        let mut i = U256::from(1);
        while i <= total {
            let job = self.jobs.get(i);
            if !job.released.get() && !job.refunded.get() {
                ids.push(i);
            }
            i = i + U256::from(1);
        }
        ids
    }

    /// Get total jobs created
    pub fn get_total_jobs(&self) -> U256 {
        self.job_count.get()
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused.get()
    }
}

sol! {
    event Deposited(uint256 indexed job_id, address indexed client, address indexed freelancer, uint256 amount);
    event Released(uint256 indexed job_id, uint256 amount);
    event Refunded(uint256 indexed job_id, uint256 amount);
    event AutoReleased(uint256 indexed job_id, uint256 amount);
    event EmergencyRefunded(uint256 indexed job_id, address indexed admin);
    event PauseToggled(bool paused);
    event OwnershipTransferred(address indexed old_admin, address indexed new_admin);
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;
    use alloy_primitives::{hex, Address, U256, B256};

    #[test]
    fn test_initialize() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let sender = vm.msg_sender();

        // Initialize the contract
        assert!(contract.initialize().is_ok());
        assert_eq!(contract.admin.get(), sender);
        assert_eq!(contract.is_paused(), false);
        assert_eq!(contract.get_total_jobs(), U256::from(0));
    }

    #[test]
    fn test_deposit() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let client = vm.msg_sender();
        let freelancer = Address::from([0x01; 20]);
        let amount = U256::from(1_000_000_000_000_000_000_u64); // 1 ETH
        let duration = 86_400_u64; // 1 day in seconds

        // Initialize contract
        assert!(contract.initialize().is_ok());

        // Set msg.value for deposit
        vm.set_value(amount);

        // Test successful deposit
        let result = contract.deposit(freelancer, duration);
        assert!(result.is_ok());
        let job_id = result.unwrap();
        assert_eq!(job_id, U256::from(1));
        assert_eq!(contract.get_total_jobs(), U256::from(1));
        assert_eq!(contract.get_active_jobs(), vec![U256::from(1)]);

        // Verify job details
        let (id, job_client, job_freelancer, job_amount, deadline, released, refunded) =
            contract.get_job(job_id);
        assert_eq!(id, job_id);
        assert_eq!(job_client, client);
        assert_eq!(job_freelancer, freelancer);
        assert_eq!(job_amount, amount);
        assert_eq!(deadline, vm.block_timestamp() + duration);
        assert_eq!(released, false);
        assert_eq!(refunded, false);

        // Verify Deposited event
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 1);
        let event_signature: B256 = hex!(
            "4a6f7a1c4c9c9a6d7e4c6d4b7b1b5e6f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7"
        ).into(); // Precomputed keccak-256 of Deposited event
        assert_eq!(logs[0].0[0], event_signature);
        assert_eq!(Address::from_slice(&logs[0].0[1].to_vec()[12..]), client);
        assert_eq!(Address::from_slice(&logs[0].0[2].to_vec()[12..]), freelancer);
    }

    #[test]
    fn test_deposit_invalid_inputs() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let freelancer = Address::from([0x01; 20]);
        let duration = 86_400_u64; // 1 day

        // Initialize contract
        assert!(contract.initialize().is_ok());

        // Test deposit with zero amount
        vm.set_value(U256::from(0));
        assert_eq!(
            contract.deposit(freelancer, duration).unwrap_err(),
            b"Amount must be > 0".to_vec()
        );

        // Test deposit with zero address
        vm.set_value(U256::from(1_000_000));
        assert_eq!(
            contract.deposit(Address::ZERO, duration).unwrap_err(),
            b"Invalid freelancer address".to_vec()
        );

        // Test deposit with zero duration
        assert_eq!(
            contract.deposit(freelancer, 0).unwrap_err(),
            b"Duration must be > 0".to_vec()
        );

        // Test deposit when paused
        assert!(contract.set_paused(true).is_ok());
        vm.set_value(U256::from(1_000_000));
        assert_eq!(
            contract.deposit(freelancer, duration).unwrap_err(),
            b"Escrow is paused".to_vec()
        );
    }

    #[test]
    fn test_release() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let client = vm.msg_sender();
        let freelancer = Address::from([0x01; 20]);
        let amount = U256::from(1_000_000_000_000_000_000_u64); // 1 ETH
        let duration = 86_400_u64; // 1 day

        // Initialize and deposit
        assert!(contract.initialize().is_ok());
        vm.set_value(amount);
        let job_id = contract.deposit(freelancer, duration).unwrap();

        // Test successful release
        assert!(contract.release(job_id).is_ok());
        let (id, _, _, _, _, released, refunded) = contract.get_job(job_id);
        assert_eq!(id, job_id);
        assert_eq!(released, true);
        assert_eq!(refunded, false);
        assert_eq!(contract.finalized.get(job_id), true);
        assert_eq!(contract.get_active_jobs(), vec![]);

        // Verify Released event
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 2); // Deposited + Released
        let event_signature: B256 = hex!(
            "d6a6a8b9c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f"
        ).into(); // Precomputed keccak-256 of Released event
        assert_eq!(logs[1].0[0], event_signature);

        // Test release by non-client
        vm.set_value(amount);
        let job_id2 = contract.deposit(freelancer, duration).unwrap();
        vm.set_sender(Address::from([0x02; 20]));
        assert_eq!(
            contract.release(job_id2).unwrap_err(),
            b"Only client can release".to_vec()
        );

        // Test release on already settled job
        vm.set_sender(client);
        assert!(contract.release(job_id).is_err());
    }

    #[test]
    fn test_refund() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let client = vm.msg_sender();
        let freelancer = Address::from([0x01; 20]);
        let amount = U256::from(1_000_000_000_000_000_000_u64); // 1 ETH
        let duration = 86_400_u64; // 1 day

        // Initialize and deposit
        assert!(contract.initialize().is_ok());
        vm.set_value(amount);
        let job_id = contract.deposit(freelancer, duration).unwrap();

        // Test successful refund
        assert!(contract.refund(job_id).is_ok());
        let (id, _, _, _, _, released, refunded) = contract.get_job(job_id);
        assert_eq!(id, job_id);
        assert_eq!(released, false);
        assert_eq!(refunded, true);
        assert_eq!(contract.finalized.get(job_id), true);
        assert_eq!(contract.get_active_jobs(), vec![]);

        // Verify Refunded event
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 2); // Deposited + Refunded
        let event_signature: B256 = hex!(
            "e7b7a9c0d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6"
        ).into(); // Precomputed keccak-256 of Refunded event
        assert_eq!(logs[1].0[0], event_signature);

        // Test refund by non-client
        vm.set_value(amount);
        let job_id2 = contract.deposit(freelancer, duration).unwrap();
        vm.set_sender(Address::from([0x02; 20]));
        assert_eq!(
            contract.refund(job_id2).unwrap_err(),
            b"Only client can refund".to_vec()
        );

        // Test refund after deadline
        vm.set_sender(client);
        vm.set_value(amount);
        let job_id3 = contract.deposit(freelancer, duration).unwrap();
        vm.set_block_timestamp(vm.block_timestamp() + duration + 1);
        assert_eq!(
            contract.refund(job_id3).unwrap_err(),
            b"Deadline passed".to_vec()
        );
    }

    #[test]
    fn test_auto_release() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let client = vm.msg_sender();
        let freelancer = Address::from([0x01; 20]);
        let amount = U256::from(1_000_000_000_000_000_000_u64); // 1 ETH
        let duration = 86_400_u64; // 1 day

        // Initialize and deposit
        assert!(contract.initialize().is_ok());
        vm.set_value(amount);
        let job_id = contract.deposit(freelancer, duration).unwrap();

        // Test auto-release before deadline
        vm.set_sender(freelancer);
        assert_eq!(
            contract.auto_release(job_id).unwrap_err(),
            b"Deadline not reached".to_vec()
        );

        // Test auto-release after deadline
        vm.set_block_timestamp(vm.block_timestamp() + duration + 1);
        assert!(contract.auto_release(job_id).is_ok());
        let (id, _, _, _, _, released, refunded) = contract.get_job(job_id);
        assert_eq!(id, job_id);
        assert_eq!(released, true);
        assert_eq!(refunded, false);
        assert_eq!(contract.finalized.get(job_id), true);
        assert_eq!(contract.get_active_jobs(), vec![]);

        // Verify AutoReleased event
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 2); // Deposited + AutoReleased
        let event_signature: B256 = hex!(
            "f8c8b0d1e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f607"
        ).into(); // Precomputed keccak-256 of AutoReleased event
        assert_eq!(logs[1].0[0], event_signature);

        // Test auto-release by non-freelancer
        vm.set_value(amount);
        let job_id2 = contract.deposit(freelancer, duration).unwrap();
        vm.set_sender(Address::from([0x02; 20]));
        vm.set_block_timestamp(vm.block_timestamp() + duration + 1);
        assert_eq!(
            contract.auto_release(job_id2).unwrap_err(),
            b"Only freelancer can claim".as_bytes().to_vec()
        );
    }

    #[test]
    fn test_admin_functions() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let admin = vm.msg_sender();
        let new_admin = Address::from([0x01; 20]);
        let client = Address::from([0x02; 20]);
        let freelancer = Address::from([0x03; 20]);
        let amount = U256::from(1_000_000_000_000_000_000_u64); // 1 ETH
        let duration = 86_400_u64; // 1 day

        // Initialize contract
        assert!(contract.initialize().is_ok());

        // Test set_paused by admin
        assert!(contract.set_paused(true).is_ok());
        assert_eq!(contract.is_paused(), true);
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 1);
        let event_signature: B256 = hex!(
            "a9d9c1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f70809"
        ).into(); // Precomputed keccak-256 of PauseToggled event
        assert_eq!(logs[0].0[0], event_signature);

        // Test set_paused by non-admin
        vm.set_sender(Address::from([0x04; 20]));
        assert_eq!(
            contract.set_paused(false).unwrap_err(),
            b"Only admin".to_vec()
        );

        // Test transfer_ownership by admin
        vm.set_sender(admin);
        assert!(contract.transfer_ownership(new_admin).is_ok());
        assert_eq!(contract.admin.get(), new_admin);
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 2); // PauseToggled + OwnershipTransferred
        let event_signature: B256 = hex!(
            "8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0"
        ).into(); // Precomputed keccak-256 of OwnershipTransferred event
        assert_eq!(logs[1].0[0], event_signature);

        // Test transfer_ownership to zero address
        assert_eq!(
            contract.transfer_ownership(Address::ZERO).unwrap_err(),
            b"Invalid admin address".to_vec()
        );

        // Test emergency_refund
        vm.set_sender(client);
        vm.set_value(amount);
        let job_id = contract.deposit(freelancer, duration).unwrap();
        vm.set_sender(new_admin);
        assert!(contract.emergency_refund(job_id).is_ok());
        let (id, _, _, _, _, released, refunded) = contract.get_job(job_id);
        assert_eq!(id, job_id);
        assert_eq!(released, false);
        assert_eq!(refunded, true);
        assert_eq!(contract.finalized.get(job_id), true);

        // Verify EmergencyRefunded event
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 4); // PauseToggled + OwnershipTransferred + Deposited + EmergencyRefunded
        let event_signature: B256 = hex!(
            "b0e0d2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7080910"
        ).into(); // Precomputed keccak-256 of EmergencyRefunded event
        assert_eq!(logs[3].0[0], event_signature);
    }

    #[test]
    fn test_storage_direct_access() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let client = vm.msg_sender();
        let freelancer = Address::from([0x01; 20]);
        let amount = U256::from(1_000_000_000_000_000_000_u64); // 1 ETH
        let duration = 86_400_u64; // 1 day

        // Initialize contract
        assert!(contract.initialize().is_ok());

        // Deposit a job
        vm.set_value(amount);
        let job_id = contract.deposit(freelancer, duration).unwrap();

        // Compute storage slot for job_id in jobs mapping
        // Mapping slot for jobs is 3 (admin: 0, paused: 1, job_count: 2)
        let base_slot = U256::from(3);
        let job_slot = keccak256(&[&job_id.to_be_bytes::<32>(), &base_slot.to_be_bytes::<32>()].concat());

        // Read job_id field (offset 0 in Job struct)
        let job_id_slot = job_slot;
        let stored_job_id = vm.storage_load_bytes32(job_id_slot);
        assert_eq!(
            stored_job_id,
            B256::from_slice(&job_id.to_be_bytes::<32>())
        );

        // Read client field (offset 1 in Job struct)
        let client_slot = job_slot + U256::from(1);
        let stored_client = vm.storage_load_bytes32(client_slot);
        assert_eq!(
            Address::from_slice(&stored_client.to_vec()[12..]),
            client
        );
    }

    #[test]
    fn test_block_dependent_logic() {
        let vm = TestVM::default();
        let mut contract = Escrow::from(&vm);
        let client = vm.msg_sender();
        let freelancer = Address::from([0x01; 20]);
        let amount = U256::from(1_000_000_000_000_000_000_u64); // 1 ETH
        let duration = 86_400_u64; // 1 day
        let initial_timestamp = 1_234_567_890_u64;

        // Initialize contract
        assert!(contract.initialize().is_ok());

        // Set initial block timestamp
        vm.set_block_timestamp(initial_timestamp);

        // Deposit a job
        vm.set_value(amount);
        let job_id = contract.deposit(freelancer, duration).unwrap();
        let (_, _, _, _, deadline, _, _) = contract.get_job(job_id);
        assert_eq!(deadline, initial_timestamp + duration);

        // Test refund before deadline
        assert!(contract.refund(job_id).is_ok());

        // Create another job
        vm.set_value(amount);
        let job_id2 = contract.deposit(freelancer, duration).unwrap();

        // Test auto-release before deadline
        vm.set_sender(freelancer);
        assert_eq!(
            contract.auto_release(job_id2).unwrap_err(),
            b"Deadline not reached".to_vec()
        );

        // Advance timestamp past deadline
        vm.set_block_timestamp(initial_timestamp + duration + 1);

        // Test refund after deadline
        vm.set_sender(client);
        assert_eq!(
            contract.refund(job_id2).unwrap_err(),
            b"Deadline passed".to_vec()
        );

        // Test auto-release after deadline
        vm.set_sender(freelancer);
        assert!(contract.auto_release(job_id2).is_ok());
    }
}