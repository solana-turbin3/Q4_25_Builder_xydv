import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Capstone } from "../target/types/capstone";
import * as crypto from "crypto";
import {
  PROGRAM_ID as TUKTUK_PROGRAM_ID,
  createTaskQueue,
  init as initTuktuk,
  taskQueueKey,
  taskQueueNameMappingKey,
  tuktukConfigKey,
} from "@helium/tuktuk-sdk";
import { Tuktuk } from "@helium/tuktuk-idls/lib/types/tuktuk";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

function hashString(input: string): Buffer {
  return crypto.createHash("sha256").update(input, "utf8").digest();
}

describe("capstone", () => {
  const provider = anchor.AnchorProvider.local("http://localhost:8899");
  anchor.setProvider(provider);

  const program = anchor.workspace.capstone as Program<Capstone>;
  const signer = provider.wallet.publicKey;
  const taskQueueName = `test-${Math.random().toString(36).substring(2, 15)}`;
  let taskQueue: anchor.web3.PublicKey;
  let tuktukProgram: Program<Tuktuk>;

  const [globalStatePda, globalStateBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("global")],
      program.programId
    );

  const [queueAuthorityPda, queueAuthorityBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("queue_authority")],
      program.programId
    );

  const [feesPda, feesBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("fees_vault")],
    program.programId
  );

  const tuktukConfig = tuktukConfigKey()[0];

  const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new anchor.web3.PublicKey(
    "BPFLoaderUpgradeab1e11111111111111111111111"
  );

  const USDC_MINT = new anchor.web3.PublicKey(
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
  );

  const programDataAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [program.programId.toBuffer()],
    BPF_LOADER_UPGRADEABLE_PROGRAM_ID
  )[0];

  before(async () => {
    tuktukProgram = await initTuktuk(provider);

    const globalState = await program.account.globalState.fetchNullable(
      globalStatePda
    );

    if (!globalState) {
      const config = await tuktukProgram.account.tuktukConfigV0.fetch(
        tuktukConfig
      );
      const nextTaskQueueId = config.nextTaskQueueId;
      taskQueue = taskQueueKey(tuktukConfig, nextTaskQueueId)[0];

      await tuktukProgram.methods
        .initializeTaskQueueV0({
          name: taskQueueName,
          minCrankReward: new anchor.BN(1_000_000),
          capacity: 1000,
          lookupTables: [],
          staleTaskAge: 10000,
        })
        .accounts({
          tuktukConfig,
          payer: signer,
          updateAuthority: signer,
          taskQueue,
          taskQueueNameMapping: taskQueueNameMappingKey(
            tuktukConfig,
            taskQueueName
          )[0],
        })
        .rpc();

      await tuktukProgram.methods
        .addQueueAuthorityV0()
        .accounts({
          payer: signer,
          queueAuthority: queueAuthorityPda,
          taskQueue,
        })
        .rpc();

      await program.methods
        .initialize()
        .accounts({ signer, taskQueue, programData: programDataAccount })
        .rpc();
    }
  });

  describe("create subscription", () => {
    it("can create a new subscription", async () => {
      const name = "turbin3 subscription";

      const [subscriptionPlanPda, subscriptionPlanBump] =
        anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("plan"), signer.toBuffer(), hashString(name)],
          program.programId
        );

      const merchantAta = await getAssociatedTokenAddress(USDC_MINT, signer);

      await program.methods
        .createSubscription({
          name,
          amount: new anchor.BN(1_000_000), // 1 USDC
          schedule: "1 * * * *",
          maxFailureCount: 3,
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

      assert.equal(subscription.merchant.toBase58(), signer.toBase58());
      assert.equal(subscription.name, name);
      assert.equal(
        subscription.amount.toString(),
        new anchor.BN(1_000_000).toString()
      );
      assert.equal(subscription.schedule, "1 * * * *");
      assert.equal(subscription.maxFailureCount, 3);
    });
  });
});
