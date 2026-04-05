use pinocchio::{error::ProgramError, Address};

use crate::error::SenshiError;

#[derive(Debug)]
pub struct Season {
    /// Account with authority over this PDA.
    pub authority: Address,
}

impl Season {
    pub const LEN: usize = 32;
    pub const DISCRIMINATOR: &'static [u8] = &[2, 0, 0, 0, 0, 0, 0, 0];

    pub fn new(authority: Address) -> Self {
        Self { authority }
    }

    /// Return a mutable `Season` reference from the given bytes.
    ///
    /// This function does not check if the data is initialized.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `Season`.
    #[inline(always)]
    pub unsafe fn load_mut_unchecked(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len()
            != Self::LEN
                .checked_sub(8)
                .ok_or(SenshiError::ArithmeticError)?
        {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(&mut *(bytes.as_mut_ptr() as *mut Self))
    }

    /// Returns the seeds for the PDA
    pub fn seeds() -> Vec<Vec<u8>> {
        vec![b"season".to_vec()]
    }

    /// Find the program address for the season account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// # Returns
    /// * `Pubkey` - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
    #[inline(always)]
    pub fn find_program_address(program_id: &Address) -> (Address, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds();
        let seeds_iter: Vec<&[u8]> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Address::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }
}
