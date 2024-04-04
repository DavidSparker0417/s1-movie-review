use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MovieRevieError { 
  #[error("Rating number must be in range 1~5")]
  InvalidRating,
  #[error("Data length exceeds the limit")]
  InvalidDataLength,
  #[error("Derivid PDA is different from passed pda")]
  InvalidPDA,
}

impl From<MovieRevieError> for ProgramError {
  fn from(e: MovieRevieError) -> Self {
    ProgramError::Custom(e as u32)
  }
}