import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaMangamonSale } from "../target/types/solana_mangamon_sale";

describe("solana-mangamon-sale", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolanaMangamonSale as Program<SolanaMangamonSale>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
