use anchor_lang::prelude::*;
use anchor_lang::system_program;
declare_id!("27kjU1HDLsU1kYudhryKg95ji2earYkWunWdyYQhbJjM");
// test game: https://www.mathplayground.com/mancala.html
#[program]
mod mancala {
    use super::*;    

    pub fn initialize_game_data(
        ctx: Context<InitializeGameData>, entry_fee: u64
    ) -> Result<()> {
        let game_data_account = &mut ctx.accounts.game_data_account;
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let signer = &mut ctx.accounts.signer;
        
        // initialize the game_data_account
        game_data_account.all_authorities.push(signer.key().clone());
        
        // initialize the chest_vault_account
        chest_vault_account.entry_fee = entry_fee;

        // set the game board to the initial state
        // the game board is an array of 0-13, where 0 is player 1's score pit and 1-6 holds player 1's pieces, 7 is player 2's score pit and 8-13 holds player 2's pieces
        // the game board is initialized to [0,4,4,4,4,4,4,0,4,4,4,4,4,4]
        chest_vault_account.game_board = [0,4,4,4,4,4,4,0,4,4,4,4,4,4];

        // initialize the score_sheet to the initial state
        chest_vault_account.score_sheet.player_one = signer.key();
        chest_vault_account.score_sheet.player_one_score = chest_vault_account.game_board[7];
        chest_vault_account.score_sheet.player_two_score = chest_vault_account.game_board[0];
        chest_vault_account.score_sheet.total_moves = 0;
        chest_vault_account.score_sheet.game_over = false;

        let cpi_context: CpiContext<'_, '_, '_, '_, system_program::Transfer<'_>> = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: chest_vault_account.to_account_info().clone(),
            },
        );
        msg!("Transfering entry fee to chest vault account {0}", entry_fee);
        system_program::transfer(cpi_context, entry_fee)?;

        Ok(())
    }

    pub fn player_joins_game(
        ctx: Context<PlayerJoinsGame>, 
    ) -> Result<()> {
        
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let signer = &mut ctx.accounts.signer;

        // if the score_sheet.player_two is not empty, then the game is already full
        if chest_vault_account.score_sheet.player_two != Pubkey::default() {
            panic!("Game is already full");
        }

        // Clone the chest_vault_account before passing it as an argument
        let chest_vault_account_info = chest_vault_account.to_account_info().clone();

        // transfer entry fee from signer to chest_vault_account
        let cpi_context: CpiContext<'_, '_, '_, '_, system_program::Transfer<'_>> = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: signer.to_account_info().clone(),
                to: chest_vault_account_info,
            },
        );

        system_program::transfer(cpi_context, chest_vault_account.entry_fee)?;

        chest_vault_account.score_sheet.player_two = signer.key();
        chest_vault_account.score_sheet.current_move = chest_vault_account.score_sheet.player_one;

        Ok(())
    }

    pub fn make_move(
        ctx: Context<MakeMove>, selected_pit: u8
    ) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let signer = &mut ctx.accounts.signer;
        
        if selected_pit == 0 || selected_pit == 7 {
            panic!("Invalid pit selection");
        }

        // if the player is not one of the players in the game, then panic
        // if the player is not the current player, then panic
        
        if chest_vault_account.score_sheet.player_one != signer.key() && chest_vault_account.score_sheet.player_two != signer.key() {
            panic!("Player is not in the game");
        }

        if chest_vault_account.score_sheet.current_move != signer.key() {
            panic!("It is not your turn");
        }

        // create a variable to hold the current player moving, if it is player one, then it is 1, if it is player two, then it is 2
        let current_player_moving: u8;

        if chest_vault_account.score_sheet.current_move == chest_vault_account.score_sheet.player_one {
            current_player_moving = 1;
        } else {
            current_player_moving = 2;
        }

        if signer.key() == chest_vault_account.score_sheet.player_one {
            // if the selected pit is not 1-6, then panic
            if selected_pit < 1 || selected_pit > 6 {
                panic!("Invalid pit selection");
            }
        } else {
            // if the selected pit is not 8-13, then panic
            if selected_pit < 8 || selected_pit > 13 {
                panic!("Invalid pit selection");
            }
        }

        // when the player moves, they select a pit to move from
        //the player's pieces are then moved counter-clockwise around the board, one piece per pit
        // if the last piece lands in the player's score pit, they get another turn
        // if the last piece lands in an empty pit on their side, they capture that piece and all the pieces in the pit directly across from it and put them in their score pit
       // 0 is player 2's score pit and 7 is player 1's score pit
        // if the last piece lands in an empty pit on their side, they capture that piece and all the pieces in the pit directly across from it and put them in their score pit, if the pit directly across from it is empty, then nothing happens

        // ///////Game Board///////////////
        // Player 2///////////////////////
        // // 13 12 11 10 9  8 //////////
        // 0 ////////////////// 7 //////
        // // 1  2  3  4  5  6 ////////
        // Player 1///////////////////

        // if the selected pit is empty, then panic
        if chest_vault_account.game_board[selected_pit as usize] == 0 {
            panic!("Selected pit is empty");
        }

        // get the number of pieces in the selected pit
        let mut pieces_in_selected_pit = chest_vault_account.game_board[selected_pit as usize];

        // set the selected pit to 0
        chest_vault_account.game_board[selected_pit as usize] = 0;

        // move the pieces counter-clockwise around the board, one piece per pit
        let mut current_pit = selected_pit + 1;
        // while there are still pieces to move
        while pieces_in_selected_pit > 0 {

            // if the current pit is 14, then set it to 0
            if current_pit == 14 {
                current_pit = 0;
            }

            // if the current pit is the opponent's score pit, then skip it
            if current_pit == 0 && current_player_moving == 1 {
                current_pit += 1;
            } else if current_pit == 7 && current_player_moving == 2 {
                current_pit += 1;
            }

            // increment the number of pieces in the current pit
            chest_vault_account.game_board[current_pit as usize] += 1;

            // decrement the number of pieces left to move
            pieces_in_selected_pit -= 1;

            // increment the current pit
            current_pit += 1;
            
        }
        let last_pit_landed_in = current_pit - 1;
        // if the last piece lands in an empty pit on their side, they capture the piece in the empty and all the pieces in the pit directly across from it and put them in their score pit (playerOne score pit = 7, playerTwo score pit = 0)
        // if the pit directly across from it is empty, then nothing happens
        // 1 matches with 13, 2 matches with 12, 3 matches with 11, 4 matches with 10, 5 matches with 9, 6 matches with 8
        if last_pit_landed_in != 0 && last_pit_landed_in != 7 {
            if current_player_moving == 1 {
                if chest_vault_account.game_board[last_pit_landed_in as usize] == 1 && chest_vault_account.game_board[14 - last_pit_landed_in as usize] != 0 && last_pit_landed_in < 7 && last_pit_landed_in > 0 {
                    chest_vault_account.game_board[7] += chest_vault_account.game_board[14 - last_pit_landed_in as usize] + 1;
                    chest_vault_account.game_board[last_pit_landed_in as usize] = 0;
                    chest_vault_account.game_board[14 - last_pit_landed_in as usize] = 0;
                }
            } else {
                if chest_vault_account.game_board[last_pit_landed_in as usize] == 1 && chest_vault_account.game_board[14 - last_pit_landed_in as usize] != 0 && last_pit_landed_in < 14 && last_pit_landed_in > 7 {
                    chest_vault_account.game_board[0] += chest_vault_account.game_board[14 - last_pit_landed_in as usize] + 1;
                    chest_vault_account.game_board[last_pit_landed_in as usize] = 0;
                    chest_vault_account.game_board[14 - last_pit_landed_in as usize] = 0;
                }
            }
        }

        // if the current_player_moving is 1 and there are no pieces in pits 1-6, then the game is over
        // if the current_player_moving is 2 and there are no pieces in pits 8-13, then the game is over
        if current_player_moving == 1 {
            if chest_vault_account.game_board[1] == 0 && chest_vault_account.game_board[2] == 0 && chest_vault_account.game_board[3] == 0 && chest_vault_account.game_board[4] == 0 && chest_vault_account.game_board[5] == 0 && chest_vault_account.game_board[6] == 0 {
                chest_vault_account.score_sheet.game_over = true;
                if chest_vault_account.game_board[7] > chest_vault_account.game_board[0] {
                    chest_vault_account.score_sheet.winner = chest_vault_account.score_sheet.player_one;
                } else if chest_vault_account.game_board[7] < chest_vault_account.game_board[0] {
                    chest_vault_account.score_sheet.winner = chest_vault_account.score_sheet.player_two;
                } else {
                    chest_vault_account.score_sheet.winner = Pubkey::default();
                }
            }
        } else if current_player_moving == 2{
            if chest_vault_account.game_board[8] == 0 && chest_vault_account.game_board[9] == 0 && chest_vault_account.game_board[10] == 0 && chest_vault_account.game_board[11] == 0 && chest_vault_account.game_board[12] == 0 && chest_vault_account.game_board[13] == 0 {
                chest_vault_account.score_sheet.game_over = true;
                if chest_vault_account.game_board[7] > chest_vault_account.game_board[0] {
                    chest_vault_account.score_sheet.winner = chest_vault_account.score_sheet.player_one;
                } else if chest_vault_account.game_board[7] < chest_vault_account.game_board[0] {
                    chest_vault_account.score_sheet.winner = chest_vault_account.score_sheet.player_two;
                } else {
                    chest_vault_account.score_sheet.winner = Pubkey::default();
                }
            }
        }
        
        // if the last piece lands in the player's score pit, they get another turn, else it is the other player's turn
        if last_pit_landed_in == 7 && current_player_moving == 1 {
            chest_vault_account.score_sheet.current_move = chest_vault_account.score_sheet.player_one;
        } else if last_pit_landed_in == 0 && current_player_moving == 2 {
            chest_vault_account.score_sheet.current_move = chest_vault_account.score_sheet.player_two;
        } else {
            if current_player_moving == 1 {
                chest_vault_account.score_sheet.current_move = chest_vault_account.score_sheet.player_two;
            } else {
                chest_vault_account.score_sheet.current_move = chest_vault_account.score_sheet.player_one;
            }
        }

        // update the score_sheet
        chest_vault_account.score_sheet.player_one_score = chest_vault_account.game_board[7];
        chest_vault_account.score_sheet.player_two_score = chest_vault_account.game_board[0];
        chest_vault_account.score_sheet.total_moves += 1;


        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let signer = &mut ctx.accounts.signer;

        // if the game is not over, then panic
        if chest_vault_account.score_sheet.game_over == false {
            panic!("Game is not over");
        }

        // if the signer is not the winner, then panic
        if chest_vault_account.score_sheet.winner != signer.key() {
            panic!("You are not the winner");
        }

        // transfer the chest reward to the signer
        let cpi_context: CpiContext<'_, '_, '_, '_, system_program::Transfer<'_>> = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: chest_vault_account.to_account_info().clone(),
                to: signer.to_account_info().clone(),
            },
        );

        // chest reward = chestVaultAccount.entry_fee * 2
        let chest_reward = chest_vault_account.entry_fee * 2;
        system_program::transfer(cpi_context, chest_reward)?;

        // remove the signer from the game_data_account.all_authorities
        let mut index = 0;
        for authority in ctx.accounts.game_data_account.all_authorities.iter() {
            if *authority == ctx.accounts.signer.key() {
                ctx.accounts.game_data_account.all_authorities.remove(index);
                break;
            }
            index += 1;
        }

        
        Ok(())
    }

    // pub fn clear_game_list (
    //     ctx: Context<ClearGameList>
    // ) -> Result<()> {
    //     let game_data_account = &mut ctx.accounts.game_data_account;
    //     let signer = &mut ctx.accounts.signer;

    //     // if the signer is not the first authority, then panic
    //     if game_data_account.all_authorities[0] != signer.key() {
    //         panic!("You are not the first authority");
    //     }

    //     // clear the game_data_account.all_authorities
    //     game_data_account.all_authorities = Vec::new();

    //     Ok(())
    // }
}


#[derive(Accounts)]
pub struct InitializeGameData<'info> {
    #[account(
        init_if_needed,
        seeds = [
            b"mancalaData",
        ],
        bump,
        payer = signer,
        space = 1000
    )]
    pub game_data_account: Account<'info, GameDataAccount>,
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
pub struct PlayerJoinsGame<'info> {
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MakeMove<'info> {
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
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

// #[derive(Accounts)]
// pub struct ClearGameList<'info> {
//     #[account(mut)]
//     pub game_data_account: Account<'info, GameDataAccount>,
//     #[account(mut)]
//     pub signer: Signer<'info>,
// }

#[account]
pub struct GameDataAccount {
    pub all_authorities: Vec<Pubkey>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct GameRecord {
    pub player_one: Pubkey,
    pub player_one_score: u8,
    pub player_two: Pubkey,
    pub player_two_score: u8,
    pub total_moves: u8,
    pub current_move: Pubkey,
    pub game_over: bool,
    pub winner: Pubkey,
}

#[account]
pub struct ChestVaultAccount{
    pub authority: Pubkey,
    pub chest_reward: u64,
    pub password: String,
    pub entry_fee: u64,
    pub score_sheet: GameRecord,
    pub game_board: [u8; 14],
}
