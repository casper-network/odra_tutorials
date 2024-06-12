#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use odra::{prelude::*, UnwrapOrRevert};
use odra::{Address, Mapping, Var};

#[odra::module(errors = Error)]
pub struct Election {
    end_block: Var<u64>,
    candidate_votes: Mapping<String, u32>,
    voters: Mapping<Address, bool>,
}

#[odra::odra_error]
pub enum Error {
    VotingEnded = 0,
    VoterAlreadyVoted = 1,
    CandidateDoesntExist = 2,
}

#[odra::module]
impl Election {
    pub fn init(&mut self, end_block: u64, candidates: Vec<String>) {
        self.end_block.set(end_block);
        for candidate in candidates.iter() {
            self.candidate_votes.set(&candidate, 0u32);
        }
    }

    pub fn vote(&mut self, candidate: String) {
        if self.env().get_block_time() > self.end_block.get_or_default() {
            self.env().revert(Error::VotingEnded);
        }

        let caller: Address = self.env().caller();

        match self.voters.get(&caller) {
            Some(_) => self.env().revert(Error::VoterAlreadyVoted),
            None => {}
        }

        let candidate_vote_count: u32 = self
            .candidate_votes
            .get(&candidate)
            .unwrap_or_revert_with(&self.env(), Error::CandidateDoesntExist);
        self.candidate_votes
            .set(&candidate, candidate_vote_count + 1);
        self.voters.set(&caller, true);
    }

    pub fn get_candidate_votes(&self, candidate: String) -> u32 {
        self.candidate_votes.get_or_default(&candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::{ElectionHostRef, ElectionInitArgs, Error};
    use odra::host::Deployer;

    #[test]
    fn vote() {
        let test_env = odra_test::env();
        let init_args = ElectionInitArgs {
            end_block: 1,
            candidates: vec!["Alice".to_string(), "Bob".to_string()],
        };
        let mut contract = ElectionHostRef::deploy(&test_env, init_args);
        // Vote
        contract.vote("Alice".to_string());

        // Validate vote count
        assert_eq!(contract.get_candidate_votes("Alice".to_string()), 1);

        // Failed Vote (VoterAlreadyVoted)
        assert_eq!(
            contract.try_vote("Bob".to_string()),
            Err(Error::VoterAlreadyVoted.into())
        );
        test_env.advance_block_time(2);
        test_env.set_caller(test_env.get_account(1));
        // Failed Vote (VotingEnded) (Implementation Error)
        /*assert_eq!(
            contract.vote("Bob".to_string()),
            Err(Error::VoterAlreadyVoted)
        );*/
    }
}
