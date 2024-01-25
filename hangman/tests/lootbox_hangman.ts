import * as anchor from "@project-serum/anchor";
import * as assert from "assert";
import { Program } from "@project-serum/anchor";
import { LootboxHangman } from "../target/types/lootbox_hangman";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";

describe("lootbox_hangman", () => {
  const program = anchor.workspace.LootboxHangman as Program<LootboxHangman>;
  const wallet = anchor.workspace.LootboxHangman.provider.wallet
  const Keypair_Secretkey = [98,78,9,165,0,44,44,82,240,225,32,88,51,13,104,18,235,23,94,254,68,202,222,75,104,206,48,189,149,62,130,128,107,41,42,111,130,192,83,78,75,36,153,84,242,169,58,148,1,20,41,115,253,49,250,41,99,178,226,70,106,132,5,102]
  const Keypair = anchor.web3.Keypair.fromSecretKey(new Uint8Array(Keypair_Secretkey));
  const hot_pubkey = Keypair.publicKey;
  console.log('hot_pubkey', hot_pubkey.toString())
  console.log('wallet', wallet.publicKey.toString())
  // PDA for the game data account
  console.log('program.programId', program.programId.toString())
  const [newGameDataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      "hangmanData",
      wallet.publicKey.toBuffer(),
    ],
    program.programId
  )
  const [chestVaultAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      "chestVault",
      wallet.publicKey.toBuffer(),
    ],
    program.programId
  );
  console.log('newGameDataAccount', newGameDataAccount.toString())
  console.log('chestVaultAccount', chestVaultAccount.toString())
  const maxAttempts = 5;
  const password = "dog";
  
  const entryFee = new anchor.BN(.5 * LAMPORTS_PER_SOL);
  const chest_reward = new anchor.BN(.05 * LAMPORTS_PER_SOL);

  it("Initialize", async () => {

    await program.methods
      .initializeLevelOne(
        chest_reward,
        password,
        maxAttempts,
        entryFee,
      ).accounts({
        newGameDataAccount: newGameDataAccount,
        signer: wallet.publicKey,
        chestVaultAccount: chestVaultAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const gameDataAccount = await program.account.gameDataAccount.fetch(
      newGameDataAccount
    )
    const new_chest_balance = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );
    const chestVaultAccountBalanceAfter = new_chest_balance.lamports;
    // get the rent in lamports for the chest vault account
    // ctx.accounts.chest_vault_account.to_account_info().rent_epoch;



    console.log('comparing these accounts', gameDataAccount.allAuthorities[0].toString(), wallet.publicKey.toString())
    // assert.strictEqual(
    //   gameDataAccount.allAuthorities[0].toString(),
    //   wallet.publicKey.toString(),
    // )

    // assert.strictEqual(
    //   chestVaultAccountBalanceAfter,
    //   expected_chest_balance,
    // )
  })

  it("Fail to withdraw as non authority", async () => {
      // get the lamport balance of the chest vault account
    const chestVaultAccountInfo = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );

    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;

    // balance of wallet before withdraw
    const walletBalanceBefore = await program.provider.connection.getBalance(hot_pubkey);

    // tx should fail because the signer is not the authority
    const tx = () => program.
      methods.withdraw()
      .accounts({
        gameDataAccount: newGameDataAccount,
        chestVaultAccount: chestVaultAccount,
        signer: hot_pubkey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([Keypair])
      .rpc();

    expect(tx).to.throw;
  
    const expected_chest_balance = chestVaultAccountBalance;
    const chestVaultAccountInfoAfter = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );
    const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;
    const walletBalanceAfter = await program.provider.connection.getBalance(hot_pubkey);
    const expected_wallet_balance = walletBalanceBefore;

    assert.strictEqual(
      chestVaultAccountBalanceAfter,
      expected_chest_balance,
    );

    assert.strictEqual(
      walletBalanceAfter,
      expected_wallet_balance,
    );
  });
  
  it("Fail to guess letter with out paying entry fee", async () => {
    // use hot wallet to try to move right
    // expect the tx to fail because the hot wallet has not executed start game yet

    const tx = () => program.methods.guessLetter("d")
      .accounts({
        gameDataAccount: newGameDataAccount,
        chestVaultAccount: chestVaultAccount,
        player: hot_pubkey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([Keypair])
      .rpc();

    expect(tx).to.throw;

  });

  it("Player Starts Game", async () => {
    
    // get the lamport balance of the chest vault account
    const chestVaultAccountInfo = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );
    // get the lamport balance of the hot wallet
    const hotWalletAccountInfo = await program.provider.connection.getAccountInfo(  
      hot_pubkey
    );
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;
    const hotWalletAccountBalance = hotWalletAccountInfo.lamports;


    await program.methods
      .playerStartsGame()
      .accounts({
        gameDataAccount: newGameDataAccount,
        chestVaultAccount: chestVaultAccount,
        signer: hot_pubkey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([Keypair])
      .rpc();

    // Fetch the game data account
    const gameDataAccount = await program.account.gameDataAccount.fetch(
      newGameDataAccount
    );
    const expected_chest_balance = chestVaultAccountBalance + (.5 * LAMPORTS_PER_SOL);
    // const expected_hot_wallet_balance = (hotWalletAccountBalance - (.5 * LAMPORTS_PER_SOL)) - (.000005 * LAMPORTS_PER_SOL);
    const expected_hot_wallet_balance = (hotWalletAccountBalance - (.5 * LAMPORTS_PER_SOL));
    const chestVaultAccountInfoAfter = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );
    const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;
    const hotWalletAccountInfoAfter = await program.provider.connection.getAccountInfo(
      hot_pubkey
    );
    const hotWalletAccountBalanceAfter = hotWalletAccountInfoAfter.lamports;
    const info = program.coder.accounts.decode(
      "ChestVaultAccount",
      chestVaultAccountInfoAfter.data
    );

    const player_vector = info.players;

    assert.strictEqual(
      player_vector[0].player.toString(),
      hot_pubkey.toString(),
    );
    assert.strictEqual(
      chestVaultAccountBalanceAfter,
      expected_chest_balance,
    );
    assert.strictEqual(
      hotWalletAccountBalanceAfter,
      expected_hot_wallet_balance,
    );
  });

  it("Guess Incorrect Letters", async () => {
    // chest vault account balance before move right
    const chestVaultAccountInfo = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;
    const letters_to_guess = ["a", "s", "t", "i"]
    for (let i = 0; i < letters_to_guess.length; i++) {
      await program.methods.guessLetter(letters_to_guess[i])
        .accounts({
          gameDataAccount: newGameDataAccount,
          chestVaultAccount: chestVaultAccount,
          player: hot_pubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([Keypair])
        .rpc();
    }

    // Fetch the game data account
    const gameDataAccount = await program.account.gameDataAccount.fetch(
      newGameDataAccount
    );

    const chestVaultAccountInfoAfter = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );

    const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;


    const info = program.coder.accounts.decode(
      "ChestVaultAccount",
      chestVaultAccountInfoAfter.data
    );
    console.log('info after second guess', info)
    const player_vector = info.players;
    const player = player_vector[0];
  

    assert.strictEqual(
      player.playerPosition,
      0,
    );

    assert.strictEqual(
      // get the correctLetters and convert it to a string removing the commas
      player.incorrectGuesses.toString().replace(/,/g, ""),
      "asti",
    );


  });

  it("Guess Letters", async () => {
    // chest vault account balance before move right
    const chestVaultAccountInfo = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;
    const letters_to_guess = ["o", "d", "g"]
    
    for (let i = 0; i < letters_to_guess.length; i++) {
      console.log('guessing letter', letters_to_guess[i])
      await program.methods.guessLetter(letters_to_guess[i])
        .accounts({
          gameDataAccount: newGameDataAccount,
          chestVaultAccount: chestVaultAccount,
          player: hot_pubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([Keypair])
        .rpc();
    }

    // Fetch the game data account
    const gameDataAccount = await program.account.gameDataAccount.fetch(
      newGameDataAccount
    );

    const chestVaultAccountInfoAfter = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );

    const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;


    const info = program.coder.accounts.decode(
      "ChestVaultAccount",
      chestVaultAccountInfoAfter.data
    );
    console.log('info after second guess', info)
    const player_vector = info.players;
    const player = player_vector[0];
  

    assert.strictEqual(
      player.playerPosition,
      3,
    );

    assert.strictEqual(
      // get the correctLetters and convert it to a string removing the commas
      player.correctLetters.toString().replace(/,/g, ""),
      "dog",
    );


  });

  

  it("Withdraw", async () => {

    // get the lamport balance of the chest vault account
    const chestVaultAccountInfo = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;

    // balance of wallet before withdraw
    const walletBalanceBefore = await program.provider.connection.getBalance(wallet.publicKey);

    const tx = await program.methods.withdraw()
      .accounts({
        gameDataAccount: newGameDataAccount,
        chestVaultAccount: chestVaultAccount,
        signer: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Fetch the game data account
    const gameDataAccount = await program.account.gameDataAccount.fetch(
      newGameDataAccount
    );

    const expected_chest_balance = null;

    const chestVaultAccountInfoAfter = await program.provider.connection.getAccountInfo(
      chestVaultAccount
    );

    // chestVaultAccountInfoAfter.lamports is null
    // should throw error TypeError: Cannot read properties of null (reading 'lamports')

    assert.throws(() => {
      chestVaultAccountInfoAfter.lamports;
    }, TypeError);

    const walletBalanceAfter = await program.provider.connection.getBalance(wallet.publicKey);

    const expected_wallet_balance = (walletBalanceBefore + (chestVaultAccountBalance - expected_chest_balance) - .000005 * LAMPORTS_PER_SOL);
    assert.strictEqual(
      gameDataAccount.allAuthorities.length,
      0,
    );
    assert.strictEqual(
      walletBalanceAfter,
      expected_wallet_balance,
    );   
  });
});