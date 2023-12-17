use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("H9nz5R89CvQfxetReei1P6W8Tag5KgBR8RtCKtNyF5zH");


#[program]
pub mod hangman {
// use solana_program::blake3::Hash;

use super::*;

    #[error_code]
    pub enum BcryptError {
        #[msg("Bcrypt error")]
        Error,
    }

    #[error_code]
    pub enum MyError {
        #[msg("Password was wrong")]
        WrongPassword,
    }
    

    pub fn initialize_level_one(ctx: Context<InitializeLevelOne>, chest_reward: u64, password: String, max_attempts: u8, entry_fee: u64) -> Result<()> {
        ctx.accounts.chest_vault_account.chest_reward = chest_reward;
        ctx.accounts.chest_vault_account.max_attempts_left = max_attempts;
        ctx.accounts.chest_vault_account.entry_fee = entry_fee;
        ctx.accounts.chest_vault_account.solved = false;
        ctx.accounts.chest_vault_account.player_list = vec![];
   
        msg!("Chest reward set to {0} lamports", chest_reward);

        // transfer the chest reward to the chest vault account
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: ctx.accounts.chest_vault_account.to_account_info().clone(),
            },
        );

        system_program::transfer(cpi_context, chest_reward)?;

        // Usually in your production code you would not print lots of text because it cost compute units.
        msg!("Game initialized, set chest reward {0}", chest_reward);
        msg!("Max attempts set to {0}", max_attempts);
        msg!("Entry fee set to {0}", entry_fee);
        msg!("Entered Password {0}", password); //"dog"
        
        ctx.accounts.chest_vault_account.password = password;

        msg!("Password set to {0}", &ctx.accounts.chest_vault_account.password);

        Ok(())
    }


    // this will be called by the player to start the game, we will pass in the pubkey of the owner of the game_data_account to pay the entry fee to
    pub fn player_starts_game(
        ctx: Context<PlayerStartsGame>, 
    ) -> Result<()> {
        let chest_vault_account     = &mut ctx.accounts.chest_vault_account;
        let secret_word_vector      = chest_vault_account.password.split("(*)(*)").collect::<Vec<&str>>();
        //if the player_position is not 0, the game has already started and we can not start it again until it is reset
        if chest_vault_account.solved == true{
            panic!("The game has already been solved!");
        }

        if chest_vault_account.max_attempts_left == 0 {
            panic!("Max attempts reached. Game Over!");
        }


        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: chest_vault_account.to_account_info().clone(),
            },
        );

        system_program::transfer(cpi_context, chest_vault_account.entry_fee)?;

        let correct_letters_as_blanks = String::from("*").repeat(secret_word_vector.len());
        
        chest_vault_account.player_list.push(GameRecord {
            player: ctx.accounts.signer.key(),
            player_score: 0,
            correct_letters: correct_letters_as_blanks,
            incorrect_letters: String::from(""),
        });

        chest_vault_account.max_attempts_left = chest_vault_account.max_attempts_left - 1;

        Ok(())
    }

   

    pub fn guess_letter(ctx: Context<GuessLetter>, letter: String, valid: bool, valid_indexes: String) -> Result<()> {
        let letter = &letter; // Take reference to avoid .clone()
        let valid_indexes = &valid_indexes; // Take reference to avoid .clone()
        let chest_vault_account = &mut *ctx.accounts.chest_vault_account;
        let chest_reward = chest_vault_account.chest_reward;
        let valid_index_vector = valid_indexes.split(",").collect::<Vec<&str>>();   // Keep this because the length is likely to be small
        let mut _incorrect_guesses = chest_vault_account.player_list[0].incorrect_letters.clone();
        let mut correct_guess_vector = chest_vault_account.player_list[0].correct_letters.split(",").collect::<Vec<&str>>();  
        let secret_word_vector = {
            chest_vault_account.password.split("(*)(*)").collect::<Vec<&str>>()
        };
        let secret_word_length = secret_word_vector.len();
    
        // if the player is not in the player_list vector, panic
        let (_player_in_vector, _player_index) = 
            match chest_vault_account.player_list.iter().position(|player| player.player == ctx.accounts.player.key()) {
            Some(index) => (true, index),
            None => panic!("Player not found!"),
        };
        if !valid && !_incorrect_guesses.contains(letter) {
            _incorrect_guesses.push_str(letter);
        }
        if valid {
            // if the letter is valid, replace the * in correct guesses vector at the corresponding indexes (ex. [1,3])
            for valid_index in valid_index_vector.iter() {
                let valid_index = valid_index.parse::<usize>().expect("Index not an usize");
                correct_guess_vector[valid_index] = letter;
            }

            chest_vault_account.player_list[_player_index].correct_letters = correct_guess_vector.join(",");
            chest_vault_account.player_list[_player_index].player_score += valid_index_vector.len() as u8;
        }
    
        // if the player has guessed all the letters correctly, transfer the chest reward to the player
        if chest_vault_account.player_list[_player_index].correct_letters.len() == secret_word_length {
            chest_vault_account.solved = true;
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.chest_vault_account.to_account_info().clone(),
                    to: ctx.accounts.player.to_account_info().clone(),
                },
            );
            system_program::transfer(cpi_context, chest_reward)?;
        }
        Ok(())
    }

    

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let game_data_account = &mut ctx.accounts.game_data_account;
        let authority = chest_vault_account.authority;
     
        // calculate rent exemption for the chest vault account
        let rent = chest_vault_account.to_account_info().rent_epoch;

        let rent_exempt_balance = chest_vault_account.to_account_info().lamports() - rent * 5000;

        // if the signer is not the creator of the chest, panic
        if ctx.accounts.signer.key() != authority {
            panic!("You are not the creator of the chest, you can not withdraw!");
        }

        **chest_vault_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= rent_exempt_balance;
        **ctx
            .accounts
            .signer
            .to_account_info()
            .try_borrow_mut_lamports()? += rent_exempt_balance;

        
        // reset the game
        chest_vault_account.player_list = vec![];

        

        for i in 0..game_data_account.all_authorities.len() {
            if game_data_account.all_authorities[i] == authority {
                game_data_account.all_authorities.remove(i);
            }
        }

        // clear game_data_account.all_authorities
        //game_data_account.all_authorities = vec![];

        Ok(())
    }
}


#[derive(Accounts)]
pub struct InitializeLevelOne<'info> {
    #[account(
        init_if_needed,
        seeds = [
            b"gameData",
            signer.key().as_ref()
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
pub struct Withdraw<'info> {
    #[account(mut)]
    pub game_data_account: Account<'info, GameDataAccount>,
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct GameDataAccount {
    pub all_authorities: Vec<Pubkey>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub  struct GameRecord {
    pub player: Pubkey,
    pub player_score: u8,
    pub correct_letters: String,
    pub incorrect_letters: String
}

#[account]
pub struct ChestVaultAccount{
    pub authority: Pubkey,
    pub chest_reward: u64,
    pub password: String,
    pub max_attempts_left: u8,
    pub entry_fee: u64,
    pub player_list: Vec<GameRecord>,
    pub solved: bool,
}

