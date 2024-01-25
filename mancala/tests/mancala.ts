import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Mancala } from "../target/types/mancala";

describe("mancala", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Mancala as Program<Mancala>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
