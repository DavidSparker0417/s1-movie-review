use borsh::{de, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    error::MovieRevieError, instruction::MovieReviewInstruction, state::MovieAccountState,
};

pub fn my_try_from_slice_unchecked<T: borsh::BorshDeserialize>(
    data: &[u8],
) -> Result<T, ProgramError> {
    let mut data_mut = data;
    match T::deserialize(&mut data_mut) {
        Ok(result) => Ok(result),
        Err(_) => Err(ProgramError::InvalidInstructionData),
    }
}
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let instruction = MovieReviewInstruction::unpack(data)?;
    match instruction {
        MovieReviewInstruction::AddReview {
            title,
            rating,
            description,
        } => add_movie_review(program_id, accounts, title, rating, description),
        MovieReviewInstruction::UpdateReview {
            title,
            rating,
            description,
        } => update_movie_review(program_id, accounts, title, rating, description),
    }
}

pub fn add_movie_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    rating: u8,
    description: String,
) -> ProgramResult {
    msg!("Adding review...");
    // check rating number
    if rating < 1 || rating > 5 {
        msg!("Invalid rating number. {}", rating);
        return Err(MovieRevieError::InvalidRating.into());
    }
    msg!("Rating number = {}", rating);
    // check passed data size
    let total_len = 1 + 1 + (4 + title.len()) + (4 + description.len());
    msg!("passed data length : {}", total_len);
    if total_len > 1000 {
        return Err(MovieRevieError::InvalidDataLength.into());
    }
    // Extract accounts
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    msg!("initializer : {}", initializer.key);
    msg!("pda account : {}", pda_account.key);
    msg!("system program : {}", system_program.key);
    // check accounts
    if !initializer.is_signer {
        return Err(ProgramError::InvalidInstructionData);
    }
    // Derive PDA
    let (pda, seed_bump) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), title.as_bytes().as_ref()],
        program_id,
    );
    msg!("Derived pda = {}", pda);
    msg!("seed bump = {}", seed_bump);
    // check derived pda
    if pda != *pda_account.key {
        msg!("Derived pda does not same with passed pda");
        return Err(MovieRevieError::InvalidPDA.into());
    }
    // calc lamports needed in creating account
    let account_len = 1000;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(account_len);
    // create pda account
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_account.key,
            lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[&[
            initializer.key.as_ref(),
            title.as_bytes().as_ref(),
            &[seed_bump],
        ]],
    )?;

    msg!("PDA created: {}", pda);
    // get the state pointer
    let mut account_data =
        my_try_from_slice_unchecked::<MovieAccountState>(&pda_account.data.borrow()).unwrap();

    // check if it is already initialized
    if account_data.is_initialized {
        msg!("Account already exists. Cannot create!");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.title = title;
    account_data.rating = rating;
    account_data.description = description;
    account_data.is_initialized = true;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn update_movie_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    rating: u8,
    description: String,
) -> ProgramResult {
    msg!("Updating review...");
    // check rating
    msg!("Checking rating ... {}", rating);
    if rating < 1 || rating > 5 {
        return Err(MovieRevieError::InvalidRating.into());
    }

    // check data lenth
    let total_len = 1 + 1 + (4 + title.len()) + (4 + description.len());
    msg!("Checking data length : {} ...", total_len);
    if total_len > 1000 {
        return Err(MovieRevieError::InvalidDataLength.into());
    }

    // extract accounts
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // check if initializer is signer
    msg!("Checing initializer ... [{}]", initializer.key);
    if !initializer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // check if program_id is owner of pda
    msg!(
        "Checking ownership of pda[{}] between program_id[{}]",
        pda_account.key,
        program_id
    );
    if pda_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Derive PDA
    let (pda, seed_bump) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), title.as_bytes().as_ref()],
        program_id,
    );

    // Check if pda equals with passed pda.
    msg!("Checking pda[{}] with passed pda[{}]", pda, pda_account.key);
    if pda != *pda_account.key {
        return Err(MovieRevieError::InvalidPDA.into());
    }

    // get account data
    let mut account_data =
        my_try_from_slice_unchecked::<MovieAccountState>(&pda_account.data.borrow()).unwrap();

    // check if account is initialized
    msg!(
        "Checking account data is initialized. [{}]",
        account_data.is_initialized
    );
    if !account_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    msg!(
        "Updating org(name:{}, rating:{}, desc:{}) to new(name:{}, rating:{}, desc:{})",
        account_data.title,
        account_data.rating,
        account_data.description,
        title,
        rating,
        description
    );
    account_data.title = title;
    account_data.rating = rating;
    account_data.description = description;
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    msg!("Succesfully updated!");
    Ok(())
}
