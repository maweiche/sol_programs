use anchor_lang::prelude::*;
use anchor_lang::system_program;
use solana_program::keccak::Hash;
declare_id!("7bp83FtVLDPGsk2vMftjHUeyNYE7LKGFJJFh2iLE36br");

#[program]
pub mod battleship_contract {
    use super::*;

    pub fn initialize_game_data(
        ctx: Context<InitializeGameData>, entry_fee: u64
    ) -> Result<()> {
        let game_data_account = &mut ctx.accounts.game_data_account;
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let signer = &mut ctx.accounts.signer;
        // initialize the game_data_account
        game_data_account.all_creators.push(signer.key().clone());
        
        // initialize the chest_vault_account
        chest_vault_account.entry_fee = entry_fee;

        // set the game board to the initial state
        // the game board is a 2d array with 20 rows and 10 columns, all values are initialized to 0
        chest_vault_account.game_board = [
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 0
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 1
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 2
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 3
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 4
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 5
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 6
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 7
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 8
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // row 9
        ];

        // initialize the score_sheet to the initial state
        chest_vault_account.score_sheet.player_one = signer.key();
        chest_vault_account.score_sheet.player_one_score = 0;
        chest_vault_account.score_sheet.player_two_score = 0;
        chest_vault_account.score_sheet.game_over = false;
        chest_vault_account.score_sheet.winner = None;

        let cpi_context: CpiContext<'_, '_, '_, '_, system_program::Transfer<'_>> = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: chest_vault_account.to_account_info().clone(),
            },
        );
        msg!("Transfering entry fee to chest vault account {0}", entry_fee);
        system_program::transfer(cpi_context, entry_fee)?;

        chest_vault_account.authority = ctx.accounts.signer.key();

        Ok(())
    }

    pub fn player_joins_game(
        ctx: Context<PlayerJoinsGame>, 
    ) -> Result<()> {
        
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let signer = &mut ctx.accounts.signer;

        // if the score_sheet.player_two is not empty, then the game is already full
        if chest_vault_account.score_sheet.player_two != Pubkey::default() {
            return Err(ErrorCode::GameIsFull.into());
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

    pub fn choose_placement(
        ctx: Context<ChoosePlacement>, selected_squares: [[u8; 10]; 5]
    ) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let signer = &mut ctx.accounts.signer;

        // if the game is already over, then panic
        if chest_vault_account.score_sheet.game_over == true {
            return Err(ErrorCode::GameIsOver.into());
        }

        if signer.key() == chest_vault_account.score_sheet.player_one {
            //  the seclected squares will be the first 5 rows of the game board
            //  replace the game board's rows 0-4 with the selected squares
            //  ex. of selected squares:
            // [
            //     [1, 0, 0, 0, 0, 0, 0, 0, 0, 3], // row 0
            //     [1, 0, 4, 0, 0, 0, 0, 0, 0, 3], // row 1
            //     [1, 0, 4, 0, 2, 2, 2, 2, 2, 0], // row 2
            //     [0, 0, 4, 0, 0, 0, 0, 0, 0, 0], // row 3
            //     [0, 0, 4, 0, 0, 0, 0, 0, 0, 0], // row 4
            // ]
            // iterate through the selected squares and replace the game board's rows 0-4
            let mut index = 0;
            for row in selected_squares.iter() {
                
                let replacement_row: [u8; 10] = row.clone();
                
                // replace the game board's row with the replacement row
                // ex. of replacement row:
                // [1, 0, 0, 0, 0, 0, 0, 0, 0, 3] // row 0
                // [1, 0, 4, 0, 0, 0, 0, 0, 0, 3] // row 1
                // [1, 0, 4, 0, 2, 2, 2, 2, 2, 0] // row 2
                // [0, 0, 4, 0, 0, 0, 0, 0, 0, 0] // row 3
                // [0, 0, 4, 0, 0, 0, 0, 0, 0, 0] // row 4
                //  ...
                chest_vault_account.game_board[index] = replacement_row;
                index += 1;
            }   
            print!("Game board: {:?}", chest_vault_account.game_board)
        } else if signer.key() == chest_vault_account.score_sheet.player_two{
            // replace the game boards rows 5-9
            let mut second_player_index = 5;
            for row in selected_squares.iter() {
                
                let replacement_row: [u8; 10] = row.clone();
                
                // replace the game board's row with the replacement row
                chest_vault_account.game_board[second_player_index] = replacement_row;

                second_player_index += 1;
            }

            
        }

        Ok(())
    }

    pub fn make_move(
        ctx: Context<MakeMove>, selected_square: [u8; 2]
    ) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;

        // if the game is already over, then panic
        if chest_vault_account.score_sheet.game_over == true {
            return Err(ErrorCode::GameIsOver.into());
        }

        if chest_vault_account.score_sheet.current_move != ctx.accounts.signer.key() {
            return Err(ErrorCode::WrongTurn.into());
        }

        let square_to_attack_row = selected_square[0]; 
        let square_to_attack_column = selected_square[1];
        
        // check if the square_to_attack is a valid square, if it's a 7, 8, or 9, then it's not a valid square
        if chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] == 7 || chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] == 8 || chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] == 9 {
            return Err(ErrorCode::InvalidMove.into());
        }
       
        // change the game board's square to 9 if it is a hit
        if chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] != 0  && ctx.accounts.signer.key() == chest_vault_account.score_sheet.player_one {
            chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] = 9;
        } else if chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] != 0  && ctx.accounts.signer.key() == chest_vault_account.score_sheet.player_two{
            // change the game board's square to 8 if it is a miss
            chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] = 8;
        } else {
            chest_vault_account.game_board[square_to_attack_row as usize][square_to_attack_column as usize] = 7;
        }

        // change the current_move to the other player
        if chest_vault_account.score_sheet.current_move == chest_vault_account.score_sheet.player_one {
            chest_vault_account.score_sheet.current_move = chest_vault_account.score_sheet.player_two;
        } else {
            chest_vault_account.score_sheet.current_move = chest_vault_account.score_sheet.player_one;
        }

        // check if the game is over
        // if rows 0-4 are all 9s, 0s, and 8s, then player two wins
        // if rows 5-9 are all 9s, 0s, and 8s, then player one wins
        let mut player_one_wins = false;
        let mut player_two_wins = false;
        let mut player_one_squares = 0;
        let mut player_two_squares = 0;
        for row in chest_vault_account.game_board.iter() {
            for square in row.iter() {
                if *square == 9 {
                    player_one_squares += 1;
                } else if *square == 8 {
                    player_two_squares += 1;
                }
            }
        }

        if player_one_squares == 14 {
            player_one_wins = true;
        } else if player_two_squares == 14 {
            player_two_wins = true;
        }

        if player_one_wins == true {
            chest_vault_account.score_sheet.game_over = true;
            chest_vault_account.score_sheet.winner = Some(chest_vault_account.score_sheet.player_one);
        } else if player_two_wins == true {
            chest_vault_account.score_sheet.game_over = true;
            chest_vault_account.score_sheet.winner = Some(chest_vault_account.score_sheet.player_two);
        }

        Ok(())
    }
    
    pub fn withdraw_loot(ctx: Context<WithdrawLoot>) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let winner = chest_vault_account.score_sheet.winner;
        // if the signer is not the creator of the chest, panic
        if ctx.accounts.signer.key() != winner.unwrap() {
            return Err(ErrorCode::NotWinner.into());
        }
        

        let chest_vault_account_info = chest_vault_account.to_account_info().clone();
        let chest_reward = chest_vault_account.entry_fee * 2;
        // issue transfer of chest_reward to the winner from the chest_vault_account
        **chest_vault_account_info.try_borrow_mut_lamports()? -= chest_reward;
            **ctx
                .accounts
                .signer
                .to_account_info()
                .try_borrow_mut_lamports()? += chest_reward;

        Ok(())
    }

    pub fn close_account(ctx: Context<CloseAccount>) -> Result<()> {
        let chest_vault_account = &mut ctx.accounts.chest_vault_account;
        let game_data_account = &mut ctx.accounts.game_data_account;
        let winner = chest_vault_account.score_sheet.winner;

        if winner == None && chest_vault_account.score_sheet.player_two != Pubkey::default(){
            return Err(ErrorCode::GameIsNotOver.into());
        }

        // if the signer is not the creator of the chest, panic
        if ctx.accounts.signer.key() != chest_vault_account.authority {
            return Err(ErrorCode::NotCreator.into());
        }
        

        // Remove the authority from game_data_account.all_creators
        game_data_account.all_creators = game_data_account.all_creators
        .iter()
        .filter(|&&i| i != chest_vault_account.authority)
        .cloned()
        .collect();

        // reset the chest_vault_account.score_sheet
        chest_vault_account.score_sheet.player_one = Pubkey::default();
        chest_vault_account.score_sheet.player_one_score = 0;
        chest_vault_account.score_sheet.player_two = Pubkey::default();
        chest_vault_account.score_sheet.player_two_score = 0;
        chest_vault_account.score_sheet.current_move = Pubkey::default();
        chest_vault_account.score_sheet.game_over = false;
        chest_vault_account.score_sheet.winner = None;

        // Close the game_data_account and transfer all lamports to the signer
        ctx.accounts.chest_vault_account.close(ctx.accounts.signer.to_account_info())?;

        Ok(())
    }
}


#[derive(Accounts)]
pub struct InitializeGameData<'info> {
    #[account(
        init_if_needed,
        seeds = [
            b"battleshipData",
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
pub struct ChoosePlacement<'info> {
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
pub struct WithdrawLoot<'info> {
    #[account(mut)]
    pub chest_vault_account: Account<'info, ChestVaultAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
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
    pub player_one: Pubkey,
    pub player_one_score: u8,
    pub player_two: Pubkey,
    pub player_two_score: u8,
    pub current_move: Pubkey,
    pub game_over: bool,
    pub winner: Option<Pubkey>,
}

#[account]
pub struct ChestVaultAccount{
    pub authority: Pubkey,
    pub chest_reward: u64,
    pub password: String,
    pub entry_fee: u64,
    pub score_sheet: GameRecord,
    pub game_board: [[u8; 10]; 10], 
}

#[error_code]
pub enum ErrorCode {
    #[msg("Player already in Game.")]
    PlayerAlreadyInGame,

    #[msg("Game is full.")]
    GameIsFull,

    #[msg("Not player's turn.")]
    WrongTurn,

    #[msg("You are not the creator of the chest.")]
    NotCreator,

    #[msg("Invalid move.")]
    InvalidMove,

    #[msg("You are not the winner")]
    NotWinner,

    #[msg("Game is not over.")]
    GameIsNotOver,

    #[msg("Game is over.")]
    GameIsOver,
}