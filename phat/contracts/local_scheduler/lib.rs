#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use ink_lang as ink;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod local_scheduler {
    use alloc::{string::String, vec::Vec};
    use ink_storage::{
        traits::{PackedLayout, SpreadAllocate, SpreadLayout},
        Mapping,
    };
    use pink_extension as pink;
    use scale::{Decode, Encode};

    #[ink(storage)]
    #[derive(SpreadAllocate, Default)]
    pub struct LocalScheduler {
        owner: AccountId,
        num_jobs: u32,
        jobs: Mapping<u32, JobConfig>,
        active_jobs: Vec<u32>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, PackedLayout, SpreadLayout)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout)
    )]
    pub struct JobConfig {
        name: String,
        cron_expr: String,
        target: AccountId,
        call: Vec<u8>,
        enabled: bool,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        BadOrigin,
        JobNotFound,
        NotChanged,
        InvalidCronExpression,
        CronExpressionNeverFire,
        InternalErrorCacheCorrupted,
        CallDataTooShort,
        FailedToCallJob,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl LocalScheduler {
        #[ink(constructor)]
        pub fn default() -> Self {
            ink_lang::utils::initialize_contract(|this: &mut Self| {
                this.owner = Self::env().caller();
                this.num_jobs = 0;
            })
        }

        #[ink(message)]
        pub fn get_num_jobs(&self) -> u32 {
            self.num_jobs
        }

        #[ink(message)]
        pub fn get_job(&self, idx: u32) -> Result<JobConfig> {
            self.ensure_job(idx)
        }

        #[ink(message)]
        pub fn get_active_jobs(&self) -> Vec<u32> {
            self.active_jobs.clone()
        }

        #[ink(message)]
        pub fn add_job(
            &mut self,
            name: String,
            cron_expr: String,
            target: AccountId,
            call: Vec<u8>,
        ) -> Result<()> {
            self.ensure_owner()?;
            Self::ensure_cron_expr(&cron_expr)?;
            if call.len() < 4 {
                return Err(Error::CallDataTooShort);
            }
            let id = self.num_jobs;
            self.num_jobs += 1;
            self.jobs.insert(
                id,
                &JobConfig {
                    name,
                    cron_expr,
                    target,
                    call,
                    enabled: true,
                },
            );
            self.active_jobs.push(id);
            Ok(())
        }

        #[ink(message)]
        pub fn set_job_cron(&mut self, id: u32, cron_expr: String) -> Result<()> {
            self.ensure_owner()?;
            Self::ensure_cron_expr(&cron_expr)?;
            let mut job = self.ensure_job(id)?;
            job.cron_expr = cron_expr;
            self.jobs.insert(id, &job);
            Ok(())
        }

        #[ink(message)]
        pub fn set_job_target(&mut self, id: u32, target: AccountId, call: Vec<u8>) -> Result<()> {
            self.ensure_owner()?;
            if call.len() < 4 {
                return Err(Error::CallDataTooShort);
            }
            let mut job = self.ensure_job(id)?;
            job.target = target;
            job.call = call;
            self.jobs.insert(id, &job);
            Ok(())
        }

        #[ink(message)]
        pub fn set_job_enabled(&mut self, id: u32, enabled: bool) -> Result<()> {
            self.ensure_owner()?;
            let mut job = self.ensure_job(id)?;
            if job.enabled == enabled {
                return Err(Error::NotChanged);
            }
            job.enabled = enabled;
            // Update active_jobs
            if job.enabled {
                self.active_jobs.push(id);
            } else {
                self.active_jobs.retain(|job| *job != id);
            }
            self.jobs.insert(id, &job);
            Ok(())
        }

        /// Gets the current job schedule
        ///
        /// Return `None` if the job doesn't exist or it's not scheduled yet. Otherwise return
        /// the next fire time and the JobConfig.
        #[ink(message)]
        pub fn get_job_schedule(&self, id: u32) -> Option<(u64, JobConfig)> {
            // TODO: should check owner?
            let job_key = generate_job_key(id);
            let job = self.ensure_job(id).ok()?;
            pink::warn!("1");

            let value = pink::ext().cache_get(&job_key)?;
            pink::warn!("2");
            let (next_ms, cron): (u64, String) = Decode::decode(&mut &value[..]).ok()?;
            pink::warn!("3");
            if cron != job.cron_expr {
                return None;
            }
            Some((next_ms, job))
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Called by a scheduler periodically
        #[ink(message)]
        pub fn poll(&self) -> Result<()> {
            let now = pink::env().block_timestamp();
            for id in &self.active_jobs {
                let job = self.jobs.get(id).expect("Active job must exist; qed.");
                if let Err(e) = self.poll_job(*id, &job, now) {
                    pink::warn!("Poll job {id} failed: {e:?}");
                }
            }
            Ok(())
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(Error::BadOrigin)
            }
        }

        fn ensure_job(&self, id: u32) -> Result<JobConfig> {
            self.jobs.get(id).ok_or(Error::JobNotFound)
        }

        fn ensure_cron_expr(expr: &String) -> Result<()> {
            use saffron::Cron;
            let cron = expr.parse::<Cron>().or(Err(Error::InvalidCronExpression))?;
            if !cron.any() {
                return Err(Error::CronExpressionNeverFire);
            }
            Ok(())
        }

        /// Polls an active job with cache-based scheduling strategy
        ///
        /// Cache storage: `job_id => { cron_expr, next_fire }`
        fn poll_job(&self, id: u32, job: &JobConfig, now_ms: u64) -> Result<()> {
            use chrono::prelude::*;
            use saffron::Cron;

            pink::debug!("Polling job {id}");
            let job_key = generate_job_key(id);
            if let Some(value) = pink::ext().cache_get(&job_key) {
                let (next_ms, cron): (u64, String) =
                    Decode::decode(&mut &value[..]).or(Err(Error::InternalErrorCacheCorrupted))?;
                // Try to trigger the job or wait until next time it fires
                if cron == job.cron_expr {
                    if now_ms >= next_ms {
                        // Trigger!
                        if let Err(e) = self.call(job) {
                            // TODO: deal with failure better?
                            pink::warn!("Failed to trigger job {id}: {e:?}");
                        }
                    } else {
                        // Wait for the next time to fire
                        return Ok(());
                    }
                }
            }
            // Update the cache
            let cron: Cron = job
                .cron_expr
                .parse()
                .expect("Cron expr checked earlier; qed.");
            let now_date = Utc.timestamp_millis(now_ms as i64);
            if let Some(next_date) = cron.next_after(now_date) {
                let next_ts = next_date.timestamp_millis() as u64;
                let value = Encode::encode(&(next_ts, &job.cron_expr));
                // TODO: handle StorageQuotaExceeded error here?
                pink::ext().cache_set(&job_key, &value);
                pink::debug!("Scheduling job {id} at timestamp {next_ts}");
            } else {
                pink::ext().cache_remove(&job_key);
                pink::debug!("Remove job {id}");
            }
            Ok(())
        }

        /// Triggers the contract call defined in the job
        fn call(&self, job: &JobConfig) -> Result<()> {
            use ink_env::call::{build_call, Call, ExecutionInput, Selector};
            // Make CallBuilder happy (Note that the call data is never less than 4 bytes)
            let mut selector = [0u8; 4];
            selector.copy_from_slice(&job.call[..4]);
            let args = PreEncodedArgs(&job.call[4..]);
            // Build CallParams
            let call_params = build_call::<pink::PinkEnvironment>()
                .call_type(
                    Call::new()
                        .callee(job.target.clone())
                        // .gas_limit(5000)
                        .transferred_value(0),
                )
                .exec_input(ExecutionInput::new(Selector::new(selector)).push_arg(args))
                .returns::<()>()
                .params();
            self.env()
                .invoke_contract(&call_params)
                .or(Err(Error::FailedToCallJob))?;
            Ok(())
        }
    }

    struct PreEncodedArgs<'a>(&'a [u8]);
    impl<'a> Encode for PreEncodedArgs<'a> {
        fn size_hint(&self) -> usize {
            self.0.len()
        }
        fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
            dest.write(self.0)
        }
    }

    pub fn generate_job_key(id: u32) -> Vec<u8> {
        Encode::encode(&(b"job", id))
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use ink::ToAccountId;
        use ink_lang as ink;

        #[ink::test]
        fn it_works() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            // Register contracts
            let hash1 = ink_env::Hash::try_from([10u8; 32]).unwrap();
            let hash2 = ink_env::Hash::try_from([20u8; 32]).unwrap();
            ink_env::test::register_contract::<LocalScheduler>(hash1.as_ref());
            ink_env::test::register_contract::<sample_oracle::SampleOracle>(hash2.as_ref());

            // Deploy Scheduler
            let mut scheduler = LocalSchedulerRef::default()
                .code_hash(hash1)
                .endowment(0)
                .salt_bytes([0u8; 0])
                .instantiate()
                .expect("failed to deploy EvmTransactor");

            // Deploy Oracle
            let mut oracle = ::sample_oracle::SampleOracleRef::default()
                .code_hash(hash2)
                .endowment(0)
                .salt_bytes([0u8; 0])
                .instantiate()
                .expect("failed to deploy SampleOracle");

            // Can add a job
            scheduler
                .add_job(
                    "job1".to_string(),
                    "* * * * *".to_string(),
                    oracle.to_account_id(),
                    hex_literal::hex!("deadbeef").to_vec(),
                )
                .expect("add job should succeed");
            assert_eq!(scheduler.get_num_jobs(), 1);
            let r = scheduler.get_job(0).unwrap();
            assert!(r.enabled);
            assert_eq!(scheduler.get_active_jobs(), vec![0]);

            // Newly added job is not scheduled yet
            assert!(scheduler.get_job_schedule(0).is_none());

            // Poll once to schedule it for the next time
            scheduler.poll().expect("first poll should succeed");
            let (next_ms, _) = scheduler.get_job_schedule(0).expect("should be triggered");
            assert_eq!(next_ms, 60_000);
        }
    }
}
