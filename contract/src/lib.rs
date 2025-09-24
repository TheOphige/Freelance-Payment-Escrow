extern crate alloc;

use stylus_sdk::prelude::*;
use alloy_primitives::{U256, Address, Uint};
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