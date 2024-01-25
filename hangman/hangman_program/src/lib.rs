use anchor_lang::prelude::*;
use anchor_lang::system_program;
use tiny_keccak::{Sha3, Hasher};

declare_id!("2whDY4jGVcLqzQYKYyfSL7sM1MU9GuxJ5kFpBen7Xnoo");

#[program]
mod lootbox_hangman {
    use super::*;    


  pub fn initialize_level_one(ctx: Context<InitializeLevelOne>, chest_reward: u64, password: String, max_attempts: u8, entry_fee: u64) -> Result<()> {
        ctx.accounts.chest_vault_account.chest_reward = chest_reward;
        ctx.accounts.chest_vault_account.max_attempts_left = max_attempts;
        ctx.accounts.chest_vault_account.entry_fee = entry_fee;
        ctx.accounts.chest_vault_account.solved = false;
        let salt = "mattisthebest";
        // let mut salt_hasher = Sha3::v256();
        // salt_hasher.update(salt.as_bytes());
        // let mut hashed_salt = [0u8; 32];
        // salt_hasher.finalize(&mut hashed_salt);

        
        
        // create a new vector to store the hashed letters
        let mut hashed_password_vector = vec![];

        //add the hashed letters to the vector

        
        
        // let password_vector = password.split("").collect::<Vec<&str>>();

        
        // iterate over the password vector and hash the letters, the vector is seperated by an empty string, so we have to skip the first and last element
        // for letter in password_vector.iter().skip(1).take(password_vector.len() - 2) {
        //     let mut hasher = Sha3::v256();
        //     let mut salt_hasher = Sha3::v256();
        //     salt_hasher.update(salt.as_bytes());
        //     let mut hashed_salt = [0u8; 32];
        //     salt_hasher.finalize(&mut hashed_salt);
        //     let hashed_salt = hex::encode(hashed_salt);
        //     hasher.update((letter.to_string() + &hashed_salt.to_string()).as_bytes());

        //     let mut hashed_password = [0u8; 32];
        
        //     // calculate the hash
        //     hasher.finalize(&mut hashed_password);
        
        //     // Convert to hexadecimal string for easier comparison
        //     let hashed_password = hex::encode(hashed_password);
        //     hashed_password_vector.push(hashed_password);
        // }
        // let _hashed_password = hashed_password_vector.join("(*)(*)");
        let _hashed_password = "d(*)(*)o(*)(*)g";
        
        // turn _hashed_password into a byte array so we can store it in the account 

        ctx.accounts.chest_vault_account.password = _hashed_password;
        // create a new vector of players
        ctx.accounts.chest_vault_account.players = vec![];
        // ctx.accounts.new_game_data_account.all_authorities.push(ctx.accounts.signer.key());
        
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: ctx.accounts.chest_vault_account.to_account_info().clone(),
            },
        );

        system_program::transfer(cpi_context, chest_reward)?;
        

        Ok(())
    }


    // this will be called by the player to start the game, we will pass in the pubkey of the owner of the game_data_account to pay the entry fee to
    pub fn player_starts_game(
        ctx: Context<PlayerStartsGame>, 
    ) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        
        //if the player_position is not 0, the game has already started and we can not start it again until it is reset
        if chest_vault_account.solved == true{
            panic!("The game has already been solved!");
        }

        if chest_vault_account.max_attempts_left == 0 {
            panic!("Max attempts reached. Game Over!");
        }

        // iterate over the players vector and check if the player is already in there, if yes and their score is above 0, panic
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
            chest_vault_account.password.split("(*)(*)").collect::<Vec<&str>>()
        };
        let secret_word_length = secret_word_vector.len();

        let item = GameRecord {
            player: ctx.accounts.signer.key(),
            player_score: 0,
            player_position: 0,
            incorrect_guesses: vec![],
            // create a vector of _ with the length of the secret word seperated by '(*)(*)'
            correct_letters: vec!["_".to_string(); secret_word_length],
        };

        chest_vault_account.players.push(item);

        chest_vault_account.max_attempts_left = chest_vault_account.max_attempts_left - 1;

        Ok(())
    }

    

    pub fn guess_letter(ctx: Context<GuessLetter>, letter: String) -> Result<()> {
        let chest_vault_account_info = ctx.accounts.chest_vault_account.to_account_info();
        let chest_vault_account = &mut *ctx.accounts.chest_vault_account;
        let chest_reward = chest_vault_account.chest_reward;
        let mut listed_player_postion = 0;
        let mut listed_incorrect_guesses = vec![];
        let mut listed_correct_guesses = vec![];
        let mut player_in_vector = false;

        // get the players position from the players vector
        for player in chest_vault_account.players.iter() {
            if player.player == ctx.accounts.player.key() {
                listed_player_postion = player.player_position as u8;
                listed_incorrect_guesses = player.incorrect_guesses.clone();
                listed_correct_guesses = player.correct_letters.clone();
            }
        }

        // if the incoming letter is already in the listed incorrect guesses vector or listed correct guesses vector, panic
        for i in 0..listed_incorrect_guesses.len() {
            if listed_incorrect_guesses[i] == letter {
                panic!("You have already guessed this letter incorrectly!");
            }
        }

        for i in 0..listed_correct_guesses.len() {
            if listed_correct_guesses[i] == letter {
                panic!("You have already guessed this letter correctly!");
            }
        }

        // check if the player is in the players vector and if not, panic
        //the players vector is a vector of objects of type GameRecord, so we have to iterate over the vector and check if the player is in there
        for player in chest_vault_account.players.iter() {
            if player.player == ctx.accounts.player.key() {
                player_in_vector = true;
            }
        }

        if player_in_vector == false {
            panic!("You are not in the players vector!");
        }

        // get the length of the secret word by seperating the string into a vector of chars and then getting the length of the vector, split the string at ever '(*)(*)'
        let secret_word_vector = {
            chest_vault_account.password.split("(*)(*)").collect::<Vec<&str>>()
        };
        let secret_word_length = secret_word_vector.len();
        
        // if listed_correct_guesses.len() is > 0 then use it to create the correct_base_vector, else use vec!["_"; secret_word_length];
        let mut correct_base_vector = listed_correct_guesses
            .len()
            .checked_sub(0)
            .map_or_else(|| vec!["_"; secret_word_length], |_| listed_correct_guesses.iter().map(AsRef::as_ref).collect());
        // take 
        let mut correct_guess = false;
        // iterate over the secret word vector and check if the letter is in the array using verify from the bcrypt crate
        for i in 0..secret_word_vector.len() {
            
            let salt = "mattisthebest";
            let mut hasher = Sha3::v256();
            let mut salt_hasher = Sha3::v256();
            salt_hasher.update(salt.as_bytes());
            let mut hashed_salt = [0u8; 32];
            salt_hasher.finalize(&mut hashed_salt);
            let hashed_salt = hex::encode(hashed_salt);

            hasher.update((letter.to_string() + &hashed_salt.to_string()).as_bytes());

            let mut hashed_password = [0u8; 32];
        
            // calculate the hash
            hasher.finalize(&mut hashed_password);
        
            // Convert to hexadecimal string for easier comparison
            let hashed_password = hex::encode(hashed_password);


            if secret_word_vector[i] == &hashed_password { 
                correct_guess = true;
                // increase the player position by 1
                // update the player position in the players vector
                for player in chest_vault_account.players.iter_mut() {
                    if player.player == ctx.accounts.player.key() {
                        player.player_score = listed_player_postion / secret_word_length as u8;
                        player.player_position = listed_player_postion + 1;
                        
                        //get the current correct letters vector and replace the _ at the correct position, then update the vector
                        correct_base_vector[i] = &letter;
                        player.correct_letters =   correct_base_vector.iter().map(|s| s.to_string()).collect();
                        
                        if player.player_position == secret_word_length as u8 {
                
                            // issue transfer of chest reward to player
                            **chest_vault_account_info.try_borrow_mut_lamports()? -= chest_reward;
                            **ctx
                                .accounts
                                .player
                                .to_account_info()
                                .try_borrow_mut_lamports()? += chest_reward;

                            chest_vault_account.max_attempts_left = 0;
                            chest_vault_account.solved = true;
                        }
                    }
                }

            }
            
        }


        if correct_guess == false {
            // update the player position in the players vector
            
            for player in chest_vault_account.players.iter_mut() {
                if player.player == ctx.accounts.player.key() {
                    player.incorrect_guesses.push(letter.to_string());
                }
            }
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
        chest_vault_account.players = vec![];

        

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
            b"hangmanData",
            signer.key().as_ref()
        ],
        bump,
        payer = signer,
        space = 1000
    )]
    pub new_game_data_account: Account<'info, GameDataAccount>,
    // This is the PDA in which we will deposit the reward SOl and
    // from where we send it back to the first player reaching the chest.
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
pub struct GameRecord {
    pub player: Pubkey,
    pub player_score: u8,
    pub player_position: u8,
    pub incorrect_guesses: Vec<String>,
    pub correct_letters: Vec<String>,
}

#[account]
pub struct ChestVaultAccount{
    pub authority: Pubkey,
    pub chest_reward: u64,
    pub password: String,
    pub max_attempts_left: u8,
    pub entry_fee: u64,
    pub players: Vec<GameRecord>,
    pub solved: bool,
}