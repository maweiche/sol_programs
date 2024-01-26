import * as assert from "assert";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { HangmanContract } from "../target/types/hangman_contract";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";

let chest_vault_data_account = null;

describe("hangman", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.HangmanContract as Program<HangmanContract>;
  const wallet = anchor.workspace.HangmanContract.provider.wallet;
  
  const hot_pubkey = wallet.publicKey;
  console.log("hot_pubkey", hot_pubkey.toString());
  console.log("wallet", wallet.publicKey.toString());
  // PDA for the game data account
  console.log("program.programId", program.programId.toString());
  async function getWalletBalance() {
    const walletBalance = await program.provider.connection.getBalance(
      wallet.publicKey
    );
    // console.log('wallet balance', walletBalance)
    return walletBalance;
  }
  getWalletBalance();
  const [newGameDataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    ["hangmanData", wallet.publicKey.toBuffer()],
    program.programId
  );
  const [chestVaultAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    ["chestVault", wallet.publicKey.toBuffer()],
    program.programId
  );
  console.log("newGameDataAccount", newGameDataAccount.toString());
  console.log("chestVaultAccount", chestVaultAccount.toString());
  const maxAttempts = 5;
  const password = "d(*)(*)o(*)(*)g".toString();

  const entryFee = new anchor.BN(0.5 * LAMPORTS_PER_SOL);
  const chest_reward = new anchor.BN(0.05 * LAMPORTS_PER_SOL);

  it("Initialize", async () => {
    try {
      await program.methods
        .initializeLevelOne(chest_reward, password, maxAttempts, entryFee)
        .accounts({
          newGameDataAccount: newGameDataAccount,
          signer: wallet.publicKey,
          chestVaultAccount: chestVaultAccount,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const gameDataAccount = await program.account.gameDataAccount.fetch(
        newGameDataAccount
      );
      const new_chest_balance =
        await program.provider.connection.getAccountInfo(chestVaultAccount);
      const chestVaultAccountBalanceAfter = new_chest_balance.lamports;
      chest_vault_data_account = new_chest_balance.data;
      // get the rent in lamports for the chest vault account
      // ctx.accounts.chest_vault_account.to_account_info().rent_epoch;

      assert.strictEqual(
        gameDataAccount.allCreators[0].toString(),
        wallet.publicKey.toString()
      );
    } catch (err) {
      console.log(err);
    }
  });

  it("Fail to withdraw as non authority", async () => {
    // get the lamport balance of the chest vault account
    const chestVaultAccountInfo =
      await program.provider.connection.getAccountInfo(chestVaultAccount);

    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;

    // balance of wallet before withdraw
    const walletBalanceBefore = await program.provider.connection.getBalance(
      hot_pubkey
    );

    // tx should fail because the signer is not the authority
    const tx = () =>
      program.methods
        .withdraw()
        .accounts({
          gameDataAccount: newGameDataAccount,
          chestVaultAccount: chestVaultAccount,
          signer: hot_pubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

    expect(tx).to.throw;

    const expected_chest_balance = chestVaultAccountBalance;
    const chestVaultAccountInfoAfter =
      await program.provider.connection.getAccountInfo(chestVaultAccount);
    const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;
    const walletBalanceAfter = await program.provider.connection.getBalance(
      hot_pubkey
    );
    const expected_wallet_balance = walletBalanceBefore;

    assert.strictEqual(chestVaultAccountBalanceAfter, expected_chest_balance);

    assert.strictEqual(walletBalanceAfter, expected_wallet_balance);
  });

  it("Fail to guess letter with out paying entry fee", async () => {
    // use hot wallet to try to move right
    // expect the tx to fail because the hot wallet has not executed start game yet

    const tx = () =>
      program.methods
        .addCorrectLetter(
          "d", //type: string
         Buffer.from([0] // type: u8
        ))
        .accounts({
          gameDataAccount: newGameDataAccount,
          chestVaultAccount: chestVaultAccount,
          player: hot_pubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

    expect(tx).to.throw;
  });

  it("Player Starts Game", async () => {
    // get the lamport balance of the chest vault account
    const chestVaultAccountInfo =
      await program.provider.connection.getAccountInfo(chestVaultAccount);
    // get the lamport balance of the hot wallet
    const hotWalletAccountInfo =
      await program.provider.connection.getAccountInfo(hot_pubkey);
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;
    const hotWalletAccountBalance = hotWalletAccountInfo.lamports;

    try {
      await program.methods
        .playerStartsGame()
        .accounts({
          gameDataAccount: newGameDataAccount,
          chestVaultAccount: chestVaultAccount,
          signer: hot_pubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      // Fetch the game data account
      const gameDataAccount = await program.account.gameDataAccount.fetch(
        newGameDataAccount
      );
      const expected_chest_balance =
        chestVaultAccountBalance + 0.5 * LAMPORTS_PER_SOL;
      // const expected_hot_wallet_balance = (hotWalletAccountBalance - (.5 * LAMPORTS_PER_SOL)) - (.000005 * LAMPORTS_PER_SOL);
      const expected_hot_wallet_balance =
        hotWalletAccountBalance - 0.5 * LAMPORTS_PER_SOL;
      const chestVaultAccountInfoAfter =
        await program.provider.connection.getAccountInfo(chestVaultAccount);
      const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;
      const hotWalletAccountInfoAfter =
        await program.provider.connection.getAccountInfo(hot_pubkey);
      const hotWalletAccountBalanceAfter = hotWalletAccountInfoAfter.lamports;
      const info = program.coder.accounts.decode(
        "ChestVaultAccount",
        chestVaultAccountInfoAfter.data
      );

      const player_vector = info.players;
      // console.log('player_vector', player_vector[0].player.toString())
      assert.strictEqual(
        player_vector[0].player.toString(),
        hot_pubkey.toString()
      );
      assert.strictEqual(chestVaultAccountBalanceAfter, expected_chest_balance);
      assert.strictEqual(
        hotWalletAccountBalanceAfter,
        expected_hot_wallet_balance
      );
    } catch (err) {
      console.log(err);
    }
  });

  it("Guess Incorrect Letters", async () => {
    // chest vault account balance before move right
    const chestVaultAccountInfo =
      await program.provider.connection.getAccountInfo(chestVaultAccount);
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;
    const letters_to_guess = ["a", "s", "t", "i"];
    for (let i = 0; i < letters_to_guess.length; i++) {
      try {
        await program.methods
          .addIncorrectLetter(letters_to_guess[i])
          .accounts({
            gameDataAccount: newGameDataAccount,
            chestVaultAccount: chestVaultAccount,
            player: hot_pubkey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
      } catch (err) {
        console.log(err);
      }
    }

    // Fetch the game data account
    const gameDataAccount = await program.account.gameDataAccount.fetch(
      newGameDataAccount
    );

    const chestVaultAccountInfoAfter =
      await program.provider.connection.getAccountInfo(chestVaultAccount);

    const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;

    const info = program.coder.accounts.decode(
      "ChestVaultAccount",
      chestVaultAccountInfoAfter.data
    );
    // console.log('info after second guess', info)
    const player_vector = info.players;

    const player = player_vector[0].player;
    const player_position = player_vector[0].playerScore;
    const incorrect_letters = player_vector[0].incorrectGuesses;
    assert.strictEqual(parseInt(player_position), 0);

    assert.strictEqual(incorrect_letters.toString().replace(/,/g, ""), "asti");
  });

  it("Guess Correct Letters & Get Pay Out", async () => {
    // chest vault account balance before move right
    const chestVaultAccountInfo =
      await program.provider.connection.getAccountInfo(chestVaultAccount);
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;
    const letters_to_guess = ["o", "g", "d"];

    for (let i = 0; i < letters_to_guess.length; i++) {
      // console.log('guessing letter', letters_to_guess[i])
      try {
        await program.methods
          .addCorrectLetter(letters_to_guess[i], Buffer.from([i]))
          .accounts({
            gameDataAccount: newGameDataAccount,
            chestVaultAccount: chestVaultAccount,
            player: hot_pubkey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
      } catch (err) {
        console.log(err);
      }
    }

    // Fetch the game data account
    const gameDataAccount = await program.account.gameDataAccount.fetch(
      newGameDataAccount
    );

    const chestVaultAccountInfoAfter =
      await program.provider.connection.getAccountInfo(chestVaultAccount);

    const chestVaultAccountBalanceAfter = chestVaultAccountInfoAfter.lamports;

    const info = program.coder.accounts.decode(
      "ChestVaultAccount",
      chestVaultAccountInfoAfter.data
    );
    // console.log('info after second guess', info)
    const player_vector = info.players;
    const player = player_vector[0].player;
    const player_score = player_vector[0].playerScore;
    // console.log('player_score', player_score)
    const player_position = player_vector[0].playerPosition;
    // console.log('player_position', player_position)
    const correct_letters = player_vector[0].correctLetters;
    // console.log('correct_letters', correct_letters)
    const incorrect_letters = player_vector[0].incorrectGuesses;

    assert.strictEqual(parseInt(player_position), 3);

    assert.strictEqual(
      // get the correctLetters and convert it to a string removing the commas
      correct_letters.toString().replace(/,/g, ""),
      "dog"
    );
  });

  it("Withdraw and Close Account", async () => {
    const [newGameDataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      ["hangmanData", wallet.publicKey.toBuffer()],
      program.programId
    );
    // get the lamport balance of the chest vault account
    const chestVaultAccountInfo =
      await program.provider.connection.getAccountInfo(chestVaultAccount);

    const wallet_balance = await program.provider.connection.getBalance(
      wallet.publicKey
    );
    const chestVaultAccountBalance = chestVaultAccountInfo.lamports;
    // console.log('chestVaultAccountBalance', chestVaultAccountBalance)
    await program.provider.connection.getBalance(wallet.publicKey);
    try {
      await program.methods
        .withdraw()
        .accounts({
          gameDataAccount: newGameDataAccount,
          chestVaultAccount: chestVaultAccount,
          signer: wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    } catch (err) {
      console.log(err);
    }

    //  this function will close the chest vault account and send the funds to the authority

    const chestVaultAccountInfoAfter =
      await program.provider.connection.getAccountInfo(chestVaultAccount);
    const wallet_balance_after = await program.provider.connection.getBalance(
      wallet.publicKey
    );

    const expected_authority_balance =
      chestVaultAccountBalance + wallet_balance - 5000;

    assert.strictEqual(wallet_balance_after, expected_authority_balance);
  });
});