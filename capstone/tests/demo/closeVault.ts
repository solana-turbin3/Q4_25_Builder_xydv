import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Capstone } from "../../target/types/capstone";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { readFileSync } from "fs";

describe("capstone", () => {
  const provider = anchor.AnchorProvider.local(
    "https://devnet.helius-rpc.com/?api-key=b9e05c06-0429-46ac-b3b4-91d56edb88ba"
  );

  anchor.setProvider(provider);

  const program = anchor.workspace.capstone as Program<Capstone>;
  const signer = provider.wallet.publicKey;

  // so that we can close this after demo
  const subscriber = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(
      JSON.parse(
        readFileSync(
          "/home/aditya/Code/Q4_25_Builder_xydv/capstone/tests/subscriber.json"
        ).toString()
      )
    )
  );

  console.log("subscriber: ", subscriber.publicKey.toBase58());

  describe("close vault", () => {
    it("user can close his vault", async () => {
      let s = await program.methods
        .closeVault()
        .accounts({
          subscriber: subscriber.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([subscriber])
        .rpc();

      console.log(`https://explorer.solana.com/tx/${s}?cluster=devnet`);
    });
  });
});
