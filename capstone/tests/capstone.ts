import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Capstone } from "../target/types/capstone";
import * as crypto from "crypto";
import {
  init as initTuktuk,
  nextAvailableTaskIds,
  taskKey,
  taskQueueKey,
  taskQueueNameMappingKey,
  tuktukConfigKey,
} from "@helium/tuktuk-sdk";
import { Tuktuk } from "@helium/tuktuk-idls/lib/types/tuktuk";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
  getAccount,
  createAssociatedTokenAccountInstruction,
  createTransferCheckedInstruction,
} from "@solana/spl-token";
import { assert, expect } from "chai";

function hashString(input: string): Buffer {
  return crypto.createHash("sha256").update(input, "utf8").digest();
}

describe("capstone", () => {
  const provider = anchor.AnchorProvider.local("http://localhost:8899");
  anchor.setProvider(provider);

  const program = anchor.workspace.capstone as Program<Capstone>;

  const signer = provider.wallet.publicKey;
  const signerKp = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array([
      100, 48, 60, 71, 86, 179, 14, 216, 64, 203, 186, 94, 205, 107, 73, 21, 42,
      136, 132, 201, 221, 76, 247, 175, 232, 102, 85, 60, 114, 27, 96, 243, 138,
      80, 193, 221, 63, 123, 195, 74, 131, 100, 205, 136, 241, 46, 231, 250, 96,
      245, 22, 151, 138, 74, 123, 28, 60, 111, 163, 102, 228, 101, 93, 191,
    ])
  );

  const subscriber1 = anchor.web3.Keypair.generate();
  const subscriber2 = anchor.web3.Keypair.generate();

  const taskQueueName = `test-${Math.random().toString(36).substring(2, 15)}`;

  console.log("Task Queue Name:", taskQueueName);

  let taskQueue: anchor.web3.PublicKey;
  let tuktukProgram: Program<Tuktuk>;
  const taskAmount = new anchor.BN(1_000_000);

  const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new anchor.web3.PublicKey(
    "BPFLoaderUpgradeab1e11111111111111111111111"
  );

  const USDC_MINT = new anchor.web3.PublicKey(
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
  );

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

  const programDataAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [program.programId.toBuffer()],
    BPF_LOADER_UPGRADEABLE_PROGRAM_ID
  )[0];

  const name = "turbin3 subscription";
  const [subscriptionPlanPda, subscriptionPlanBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("plan"), signer.toBuffer(), hashString(name)],
      program.programId
    );

  const merchantAta = getAssociatedTokenAddressSync(USDC_MINT, signer);

  const subscriber1Ata = getAssociatedTokenAddressSync(
    USDC_MINT,
    subscriber1.publicKey
  );
  const subscriber2Ata = getAssociatedTokenAddressSync(
    USDC_MINT,
    subscriber2.publicKey
  );

  const [subscriber1SubscriptionPda] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("subscription"),
        subscriber1.publicKey.toBuffer(),
        subscriptionPlanPda.toBuffer(),
      ],
      program.programId
    );

  const [subscriber1VaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), subscriber1.publicKey.toBuffer()],
    program.programId
  );

  before(async () => {
    tuktukProgram = await initTuktuk(provider);

    await provider.connection.requestAirdrop(
      subscriber1.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );

    await provider.connection.requestAirdrop(
      subscriber2.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );

    await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          signer,
          subscriber1Ata,
          subscriber1.publicKey,
          USDC_MINT,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        ),
        createTransferCheckedInstruction(
          merchantAta,
          USDC_MINT,
          subscriber1Ata,
          signer,
          100000000,
          6
        ),
        createAssociatedTokenAccountInstruction(
          signer,
          subscriber2Ata,
          subscriber2.publicKey,
          USDC_MINT,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        ),
        createTransferCheckedInstruction(
          merchantAta,
          USDC_MINT,
          subscriber2Ata,
          signer,
          100000000,
          6
        )
      ),
      [signerKp]
    );

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
    } else {
      taskQueue = globalState.taskQueue;
    }
  });

  describe("create subscription", () => {
    it("merchant can create a new subscription with correct details", async () => {
      await program.methods
        .createSubscription({
          name,
          amount: taskAmount,
          interval: new anchor.BN(120),
          maxFailureCount: 1,
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
      assert.equal(
        subscription.interval.toString(),
        new anchor.BN(120).toString()
      );
      assert.equal(subscription.maxFailureCount, 1);
    });

    it("should fail to create subscription with zero amount", async () => {
      const newPlanName = "zero amount test";

      const [testPlanPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("plan"), signer.toBuffer(), hashString(newPlanName)],
        program.programId
      );

      try {
        await program.methods
          .createSubscription({
            name: newPlanName,
            amount: new anchor.BN(0),
            interval: new anchor.BN(100),
            maxFailureCount: 1,
          })
          .accountsStrict({
            merchant: signer,
            mint: USDC_MINT,
            subscriptionPlan: testPlanPda,
            feesVault: feesPda,
            globalState: globalStatePda,
            merchantAta: merchantAta,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          })
          .rpc();
        assert.fail("Transaction should have failed with InvalidAmount");
      } catch (err) {
        expect(err.error.errorCode.code).to.include("InvalidAmount");
      }
    });

    it("should fail to create subscription with no name", async () => {
      const newPlanName = "";

      const [testPlanPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("plan"), signer.toBuffer(), hashString(newPlanName)],
        program.programId
      );

      try {
        await program.methods
          .createSubscription({
            name: newPlanName,
            amount: taskAmount,
            interval: new anchor.BN(100),
            maxFailureCount: 1,
          })
          .accountsStrict({
            merchant: signer,
            mint: USDC_MINT,
            subscriptionPlan: testPlanPda,
            feesVault: feesPda,
            globalState: globalStatePda,
            merchantAta: merchantAta,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          })
          .rpc();
        assert.fail("Transaction should have failed with InvalidName");
      } catch (err) {
        expect(err.error.errorCode.code).to.include("InvalidName");
      }
    });
  });

  describe("subscribe", () => {
    it("user can subscribe to a subscription", async () => {
      const taskQueueAcc = await tuktukProgram.account.taskQueueV0.fetch(
        taskQueue
      );

      const nextTask = nextAvailableTaskIds(
        taskQueueAcc.taskBitmap,
        1,
        false
      )[0];

      await program.methods
        .subscribe()
        .accountsPartial({
          subscriber: subscriber1.publicKey,
          subscriberAta: subscriber1Ata,
          subscriptionPlan: subscriptionPlanPda,
          mint: USDC_MINT,
          task: taskKey(taskQueue, nextTask)[0],
          globalState: globalStatePda,
          taskQueue,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([subscriber1])
        .rpc();

      const userSubs = await program.account.userSubscription.fetch(
        subscriber1SubscriptionPda
      );
      assert.isTrue(
        userSubs.status.active !== undefined,
        "Subscription status should be Active"
      );
    });

    it("should fail to subscribe to the same plan twice", async () => {
      const taskQueueAcc = await tuktukProgram.account.taskQueueV0.fetch(
        taskQueue
      );
      const nextTask = nextAvailableTaskIds(
        taskQueueAcc.taskBitmap,
        1,
        false
      )[0];

      try {
        await program.methods
          .subscribe()
          .accountsPartial({
            subscriber: subscriber1.publicKey,
            subscriberAta: subscriber1Ata,
            subscriptionPlan: subscriptionPlanPda,
            mint: USDC_MINT,
            task: taskKey(taskQueue, nextTask)[0],
            globalState: globalStatePda,
            taskQueue,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([subscriber1])
          .rpc();
        assert.fail(
          "Subscription should have failed because the PDA is already initialized"
        );
      } catch (err) {
        expect(err.transactionMessage).to.include("already in use");
      }
    });
  });

  describe("cancel subscription", () => {
    it("subscriber1 can cancel a subscription", async () => {
      console.log("\nwaiting for tuktuk to charge for one cycle...\n");
      await new Promise((resolve) => setTimeout(resolve, 5000));

      const userSubs = await program.account.userSubscription.fetch(
        subscriber1SubscriptionPda
      );

      const task = taskKey(taskQueue, userSubs.nextTaskId)[0];

      await program.methods
        .cancelSubscription()
        .accountsPartial({
          subscriber: subscriber1.publicKey,
          userSubscription: subscriber1SubscriptionPda,
          taskQueue,
          tokenProgram: TOKEN_PROGRAM_ID,
          task,
        })
        .signers([subscriber1])
        .rpc();

      const cancelledSubscription =
        await program.account.userSubscription.fetchNullable(
          subscriber1SubscriptionPda
        );

      expect(cancelledSubscription).to.be.null;
    });
  });

  describe("close vault", () => {
    it("user can close his vault", async () => {
      await program.methods
        .closeVault()
        .accounts({
          subscriber: subscriber1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([subscriber1])
        .rpc({ skipPreflight: true });
    });
  });

  it("subscriber2 cannot close subscriber1's vault (Wrong Signer)", async () => {
    const [subscriber1VaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), subscriber1.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .closeVault()
        .accountsStrict({
          subscriber: subscriber2.publicKey,
          mint: USDC_MINT,
          subscriberAta: subscriber2Ata,
          subscriberVault: subscriber1VaultPda,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([subscriber2])
        .rpc();
      assert.fail(
        "Transaction should have failed due to constraint violation (wrong signer)"
      );
    } catch (err) {
      expect(err.error.errorCode.code).to.include("AccountNotInitialized");
    }
  });
});
