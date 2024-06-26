use borsh::{de, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    error::MovieRevieError,
    instruction::MovieReviewInstruction,
    state::{MovieAccountState, MovieComment, MovieCommentCounter},
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
        MovieReviewInstruction::AddComment { comment } => {
            add_comment(program_id, accounts, comment)
        }
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
    let account_len: usize = 1000;
    if MovieAccountState::get_account_size(title.clone(), description.clone()) > account_len {
        msg!("Data length is larger than 1000 bytes");
        return Err(MovieRevieError::InvalidDataLength.into());
    }

    // Extract accounts
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let pda_counter = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    msg!("initializer : {}", initializer.key);
    msg!("pda account : {}", pda_account.key);
    msg!("system program : {}", system_program.key);
    // check accounts
    if !initializer.is_signer {
        return Err(ProgramError::InvalidInstructionData);
    }
    // Derive PDA counter
    msg!("Create comment counter.");

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
    account_data.discriminator = MovieAccountState::DISCRIMINATOR.to_string();
    account_data.reviewer = *initializer.key;
    account_data.title = title;
    account_data.rating = rating;
    account_data.description = description;
    account_data.is_initialized = true;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    // Create Comment counter
    msg!("create comment counter");
    // Derive counter PDA
    let (counter, counter_bump) =
        Pubkey::find_program_address(&[pda.as_ref(), "comment".as_ref()], program_id);
    // calculate the lamport
    let rent = Rent::get()?;
    let counter_rent_lamports = rent.minimum_balance(MovieCommentCounter::SIZE);
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            &counter,
            counter_rent_lamports,
            MovieCommentCounter::SIZE.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_counter.clone(),
            system_program.clone(),
        ],
        &[&[pda.as_ref(), "comment".as_ref(), &[counter_bump]]],
    )?;
    msg!("Comment counter created");
    let mut counter_data =
        my_try_from_slice_unchecked::<MovieCommentCounter>(&pda_counter.data.borrow()).unwrap();
    msg!("Checking if counter account is already initialized");
    if counter_data.is_initialized {
       msg!("Counter account is alreay initialized") ;
       return Err(ProgramError::AccountAlreadyInitialized)
    }
    counter_data.discriminator = MovieCommentCounter::DISRIMINATOR.to_string();
    counter_data.counter = 0;
    counter_data.is_initialized = true;
    msg!("comment count: {}", counter_data.counter);
    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;
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

pub fn add_comment(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    comment: String,
) -> ProgramResult {
    msg!("Add comment. '{}'", comment);
    // Extract accounts
    let iter = &mut accounts.iter();
    let commenter = next_account_info(iter)?;
    let pda_review = next_account_info(iter)?;
    let pda_counter = next_account_info(iter)?;
    let pda_comment = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    msg!("Commenter: {}", commenter.key);
    msg!("PDA Review: {}", pda_review.key);
    msg!("PDA Counter: {}", pda_counter.key);
    msg!("PDA Comment: {}", pda_comment.key);
    // check accounts
    if !commenter.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Get counter
    let mut counter_data =
        my_try_from_slice_unchecked::<MovieCommentCounter>(&pda_counter.data.borrow_mut()).unwrap();
    msg!("Counter = {}", counter_data.counter);
    // Drive PDA
    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            pda_review.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
        ],
        program_id,
    );
    // check derived pda with passed pda
    if pda != *pda_comment.key {
        msg!("Invalid seed for PDA. derived pda = {}", pda);
        return Err(MovieRevieError::InvalidPDA.into());
    }

    // Calculate lamports
    let account_len = MovieComment::get_account_size(comment.clone());
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(account_len);
    msg!("Lamports for creating comment account is {}", lamports);
    // Create account data
    invoke_signed(
        &system_instruction::create_account(
            commenter.key,
            &pda,
            lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            commenter.clone(),
            pda_comment.clone(),
            system_program.clone(),
        ],
        &[&[
            pda_review.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    msg!("Created Comment Account.");
    let mut comment_data =
        my_try_from_slice_unchecked::<MovieComment>(&pda_comment.data.borrow()).unwrap();

    msg!("Checking if comment account is already initialized");
    if comment_data.is_initialized {
        msg!("Account already initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    comment_data.discriminator = MovieComment::DISRIMINATOR.to_string();
    comment_data.commenter = *commenter.key;
    comment_data.review = *pda_review.key;
    comment_data.comment = comment;
    comment_data.is_initialized = true;
    comment_data.serialize(&mut &mut pda_comment.data.borrow_mut()[..])?;

    msg!("Comment Count: {}", counter_data.counter);
    counter_data.counter += 1;
    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    Ok(())
}
