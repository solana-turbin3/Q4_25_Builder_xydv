import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Capstone } from "../../target/types/capstone";
import * as crypto from "crypto";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

function hashString(input: string): Buffer {
  return crypto.createHash("sha256").update(input, "utf8").digest();
}

describe("capstone", () => {
  const provider = anchor.AnchorProvider.local(
    "https://devnet.helius-rpc.com/?api-key=b9e05c06-0429-46ac-b3b4-91d56edb88ba"
  );

  anchor.setProvider(provider);

  const program = anchor.workspace.capstone as Program<Capstone>;
  const signer = provider.wallet.publicKey;

  const [globalStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("global")],
    program.programId
  );

  const [feesPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("fees_vault")],
    program.programId
  );

  const USDC_MINT = new anchor.web3.PublicKey(
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" // for devnet
  );

  const name = `turbin3 subscription ${Date.now()}`; // new name for each

  const [subscriptionPlanPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("plan"), signer.toBuffer(), hashString(name)],
    program.programId
  );

  const merchantAta = getAssociatedTokenAddressSync(USDC_MINT, signer);

  describe("create subscription", () => {
    it("can create a new subscription", async () => {
      await program.methods
        .createSubscription({
          name,
          amount: new anchor.BN(1_000_000), // 1 USDC
          interval: new anchor.BN(120),
          maxFailureCount: 2,
        })
        .accountsStrict({
          merchant: signer,
          mint: USDC_MINT,
          subscriptionPlan: subscriptionPlanPda,
          feesVault: feesPda,
          globalState: globalStatePda,
          merchantAta: merchantAta,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .rpc();

      const subscription = await program.account.subscriptionPlan.fetch(
        subscriptionPlanPda
      );

      console.log("PLAN: ", subscriptionPlanPda.toBase58());

      assert.equal(subscription.merchant.toBase58(), signer.toBase58());
      assert.equal(subscription.name, name);
      assert.equal(
        subscription.amount.toString(),
        new anchor.BN(1_000_000).toString()
      );
      assert.equal(
        subscription.interval.toString(),
        new anchor.BN(120).toString()
      );
      assert.equal(subscription.maxFailureCount, 2);
    });
  });
});
