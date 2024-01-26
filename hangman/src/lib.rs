use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("89qhdKLthboXkjWT8icXditnmnSZAxCtqRXcAEm8Kb5i");

#[program]
mod hangman_contract {
    use super::*;

    pub fn initialize_level_one(ctx: Context<InitializeLevelOne>, chest_reward: u64, password: String, max_attempts: u8, entry_fee: u64) -> Result<()> {
        ctx.accounts.chest_vault_account.chest_reward = chest_reward;
        ctx.accounts.chest_vault_account.max_attempts_left = max_attempts;
        ctx.accounts.chest_vault_account.entry_fee = entry_fee;
        ctx.accounts.chest_vault_account.players = vec![];
        ctx.accounts.chest_vault_account.password = password;
        ctx.accounts.chest_vault_account.creator = ctx.accounts.signer.key();
        ctx.accounts.new_game_data_account.all_creators.push(ctx.accounts.signer.key());

        let cpi_context: CpiContext<'_, '_, '_, '_, system_program::Transfer<'_>> = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: ctx.accounts.chest_vault_account.to_account_info().clone(),
            },
        );
        msg!("Password is {0}", ctx.accounts.chest_vault_account.password);
        msg!("All creators are {:?}", ctx.accounts.new_game_data_account.all_creators);
        system_program::transfer(cpi_context, chest_reward)?;

        Ok(())
    }

    pub fn player_starts_game(
        ctx: Context<PlayerStartsGame>, 
    ) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;

        if chest_vault_account.max_attempts_left == 0 {
            panic!("Max attempts reached. Game Over!");
        }

        for player in chest_vault_account.players.iter() {
            if player.player == ctx.accounts.signer.key() && player.player_score > 0 {
                panic!("You have already played and your score is {0}. You can not play again!", player.player_score);
            }
        }

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: chest_vault_account.to_account_info().clone(),
            },
        );

        system_program::transfer(cpi_context, chest_vault_account.entry_fee)?;

        let secret_word_vector = {
            &chest_vault_account.password.split("m2w3").collect::<Vec<&str>>()
        };
        let secret_word_length = secret_word_vector.len();

        let item = GameRecord {
            player: ctx.accounts.signer.key(),
            player_score: 0,
            player_position: 0,
            incorrect_guesses: vec![],
            correct_letters: vec!["_".to_string(); secret_word_length],
            is_winner: false,
        };

        chest_vault_account.players.push(item);

        chest_vault_account.max_attempts_left = chest_vault_account.max_attempts_left - 1;

        Ok(())
    }

    pub fn get_chest_reward(ctx: Context<GetChestReward>) -> Result<()> {
        let mut player_in_vector: bool = false;
        let chest_vault_account_info: &AccountInfo<'_> = &ctx.accounts.chest_vault_account.to_account_info();
        let chest_vault_account: &mut ChestVaultAccount = &mut *ctx.accounts.chest_vault_account;

        for player in chest_vault_account.players.iter() {
            if player.player == ctx.accounts.player.key() {
                player_in_vector = true;
            }
        }
        if player_in_vector == false {
            panic!("You have not started the game yet!");
        }
        
        let chest_reward = chest_vault_account.chest_reward;
        let player_index = chest_vault_account.players.iter().position(|x| x.player == ctx.accounts.player.key()).unwrap();

        if chest_vault_account.players[player_index].is_winner == false {
            panic!("You have not solved the puzzle yet!");
        }

        msg!("The listed player position is {0}", chest_vault_account.players[player_index].player_position);
        msg!("The listed incorrect guesses are {:?}", chest_vault_account.players[player_index].incorrect_guesses);
        msg!("The listed correct guesses are {:?}", chest_vault_account.players[player_index].correct_letters);
        let secret_word_vector = {
            &chest_vault_account.password.split("m2w3").collect::<Vec<&str>>()
        };
        let secret_word_length = secret_word_vector.len();
        if chest_vault_account.players[player_index].player_position == secret_word_length as u8 {
            
            // issue transfer of chest reward to player
            **chest_vault_account_info.try_borrow_mut_lamports()? -= chest_reward;
            **ctx
                .accounts
                .player
                .to_account_info()
                .try_borrow_mut_lamports()? += chest_reward;

            chest_vault_account.max_attempts_left = 0;
        }

        Ok(())
    }

    pub fn add_correct_letter(ctx: Context<AddCorrectLetter>, letter: String, indexes: Vec<u8>) -> Result<()> {
        let mut player_in_vector: bool = false;
        let chest_vault_account: &mut ChestVaultAccount = &mut *ctx.accounts.chest_vault_account;
        for player in chest_vault_account.players.iter() {
            if player.player == ctx.accounts.player.key() {
                player_in_vector = true;
            }
        }
        if player_in_vector == false {
            panic!("You have not started the game yet!");
        }
        
        let player_index = chest_vault_account.players.iter().position(|x| x.player == ctx.accounts.player.key()).unwrap();
        
        msg!("The listed player position is {0}", chest_vault_account.players[player_index].player_position);
        msg!("The listed incorrect guesses are {:?}", chest_vault_account.players[player_index].incorrect_guesses);
        msg!("The listed correct guesses are {:?}", chest_vault_account.players[player_index].correct_letters);
        // if the incoming letter is already in the listed incorrect guesses vector or listed correct guesses vector, panic
        for i in 0..chest_vault_account.players[player_index].incorrect_guesses.len() {
            if chest_vault_account.players[player_index].incorrect_guesses[i] == letter {
                panic!("You have already guessed this letter incorrectly!");
            }
        }

        for i in 0..chest_vault_account.players[player_index].correct_letters.len() {
            if chest_vault_account.players[player_index].correct_letters[i] == letter {
                panic!("You have already guessed this letter correctly!");
            }
        }
        let secret_word_vector = {
            &chest_vault_account.password.split("m2w3").collect::<Vec<&str>>()
        };
        let secret_word_length = secret_word_vector.len();
        msg!("The secret word vector is {0}", String::from(&chest_vault_account.password));
       
        for i in 0..indexes.len() {
            msg!("Adding {0} into the correct letters vector at index {1}", letter, indexes[i]);
            
            let index_to_add = indexes[i] as usize;

            chest_vault_account.players[player_index].player_position = chest_vault_account.players[player_index].player_position + 1;
            chest_vault_account.players[player_index].player_score = chest_vault_account.players[player_index].player_position % secret_word_length as u8;        
            chest_vault_account.players[player_index].correct_letters[index_to_add] = letter.to_string();
            
            msg!("Current player position is {0}", chest_vault_account.players[player_index].player_position);
            
            if chest_vault_account.players[player_index].player_position == secret_word_length as u8 {
            
                chest_vault_account.players[player_index].is_winner = true;
            }
            
        }
        msg!("Correct Letters array is now {:?}", chest_vault_account.players[player_index].correct_letters);
        Ok(())
    }

    pub fn add_incorrect_letter(ctx: Context<AddIncorrectLetter>, letter: String) -> Result<()> {
        let mut player_in_vector = false;
        let chest_vault_account = &mut *ctx.accounts.chest_vault_account;
        // check if the player is in the players vector and if not, panic
        //the players vector is a vector of objects of type GameRecord, so we have to iterate over the vector and check if the player is in there
        for player in chest_vault_account.players.iter() {
            if player.player == ctx.accounts.player.key() {
                player_in_vector = true;
            }
        }
        if player_in_vector == false {
            panic!("You have not started the game yet!");
        }
        
        let player_index = chest_vault_account.players.iter().position(|x| x.player == ctx.accounts.player.key()).unwrap();
        
        msg!("The listed player position is {0}", chest_vault_account.players[player_index].player_position);
        msg!("The listed incorrect guesses are {:?}", chest_vault_account.players[player_index].incorrect_guesses);
        msg!("The listed correct guesses are {:?}", chest_vault_account.players[player_index].correct_letters);
        // if the incoming letter is already in the listed incorrect guesses vector or listed correct guesses vector, panic
        for i in 0..chest_vault_account.players[player_index].incorrect_guesses.len() {
            if chest_vault_account.players[player_index].incorrect_guesses[i] == letter {
                panic!("You have already guessed this letter incorrectly!");
            }
        }

        for i in 0..chest_vault_account.players[player_index].correct_letters.len() {
            if chest_vault_account.players[player_index].correct_letters[i] == letter {
                panic!("You have already guessed this letter correctly!");
            }
        }
       
        chest_vault_account.players[player_index].incorrect_guesses.push(letter);
        msg!("Incorrect Guesses array is now {:?}", chest_vault_account.players[player_index].incorrect_guesses);
      
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let game_data_account = &mut ctx.accounts.game_data_account;
        
        let authority = chest_vault_account.creator;            

        // if the signer is not the creator of the chest, panic
        if ctx.accounts.signer.key() != authority {
            panic!("You are not the creator of the chest, you can not withdraw!");
        }
        
        // reset the game
        chest_vault_account.players = vec![];

        // Remove the authority from game_data_account.all_creators
        game_data_account.all_creators = game_data_account.all_creators
        .iter()
        .filter(|&&i| i != ctx.accounts.signer.key())
        .cloned()
        .collect();


        // Close the game_data_account and transfer all lamports to the signer
        ctx.accounts.chest_vault_account.close(ctx.accounts.signer.to_account_info())?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct InitializeLevelOne<'info> {
    #[account(
        init_if_needed,
        seeds = [
            b"hangmanData",
        ],
        bump,
        payer = signer,
        space = 1000
    )]
    pub new_game_data_account: Account<'info, GameDataAccount>,
    #[account(
        init_if_needed,
        seeds = [
            b"chestVault",
            signer.key().as_ref()
        ],
        bump,
        payer = signer,
        space = 1000
    )]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlayerStartsGame<'info> {
    #[account(mut)]
    pub game_data_account: Account<'info, GameDataAccount>,
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GuessLetter<'info> {
    #[account(mut)]
    pub game_data_account: Account<'info, GameDataAccount>,
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddCorrectLetter<'info> {
    #[account(mut)]
    pub game_data_account: Account<'info, GameDataAccount>,
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddIncorrectLetter<'info> {
    #[account(mut)]
    pub game_data_account: Account<'info, GameDataAccount>,
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetChestReward<'info> {
    #[account(mut)]
    pub game_data_account: Account<'info, GameDataAccount>,
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub game_data_account: Account<'info, GameDataAccount>,
    #[account(mut, close = signer)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct GameDataAccount {
    pub all_creators: Vec<Pubkey>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct GameRecord {
    pub player: Pubkey,
    pub player_score: u8,
    pub player_position: u8,
    pub incorrect_guesses: Vec<String>,
    pub correct_letters: Vec<String>,
    pub is_winner: bool,
}

#[account]
pub struct ChestVaultAccount{
    pub creator: Pubkey,
    pub chest_reward: u64,
    pub password: String,
    pub max_attempts_left: u8,
    pub entry_fee: u64,
    pub players: Vec<GameRecord>,
}