import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Capstone } from "../target/types/capstone";
import * as crypto from "crypto";
import {
  init as initTuktuk,
  nextAvailableTaskIds,
  taskKey,
} from "@helium/tuktuk-sdk";
import { Tuktuk } from "@helium/tuktuk-idls/lib/types/tuktuk";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { readFileSync } from "fs";

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

  console.log("MERCHANT: ", signer.toBase58());
  console.log("SUBSCRIBER: ", subscriber.publicKey.toBase58());

  const taskQueueName = `test-${Math.random().toString(36).substring(2, 15)}`;

  console.log("TASK QUEUE NAME: ", taskQueueName);

  let taskQueue: anchor.web3.PublicKey;
  let tuktukProgram: Program<Tuktuk>;

  const [globalStatePda, globalStateBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("global")],
      program.programId
    );

  const [feesPda, feesBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("fees_vault")],
    program.programId
  );

  const USDC_MINT = new anchor.web3.PublicKey(
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" // for devnet
  );

  const name = `turbin3 subscription ${Math.random()
    .toString(36)
    .substring(2, 15)}}`; // new name for each

  const [subscriptionPlanPda, subscriptionPlanBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("plan"), signer.toBuffer(), hashString(name)],
      program.programId
    );

  const merchantAta = getAssociatedTokenAddressSync(USDC_MINT, signer);

  before(async () => {
    const globalState = await program.account.globalState.fetchNullable(
      globalStatePda
    );

    console.log("GLOBAL STATE: ", globalStatePda.toBase58());

    // @ts-ignore
    tuktukProgram = await initTuktuk(provider);
    taskQueue = globalState.taskQueue;

    console.log("TASK QUEUE: ", taskQueue.toBase58());
  });

  describe("create subscription", () => {
    it("can create a new subscription", async () => {
      await program.methods
        .createSubscription({
          name,
          amount: new anchor.BN(1_000_000), // 1 USDC
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

      console.log(
        "SUBSCRIPTION PLAN CREATED: ",
        subscriptionPlanPda.toBase58()
      );

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
  });

  describe("subscribe", () => {
    it("user can subscribe to a subscription", async () => {
      const [userSubscriptionPda] =
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("subscription"),
            subscriber.publicKey.toBuffer(),
            subscriptionPlanPda.toBuffer(),
          ],
          program.programId
        );

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
          userSubscription: userSubscriptionPda,
          subscriber: subscriber.publicKey,
          subscriptionPlan: subscriptionPlanPda,
          mint: USDC_MINT,
          task: taskKey(taskQueue, nextTask)[0],
          globalState: globalStatePda,
          taskQueue,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([subscriber])
        .rpc();

      console.log(
        "SUBSCRIBED TO PLAN, SUBSCRIPTION ACCOUNT: ",
        userSubscriptionPda.toBase58()
      );
    });
  });

  describe("cancel subscription", () => {
    it("user can cancel a subscription", async () => {
      console.log("WAITING FOR TUKTUK TO DEDUCT FROM VAULT...");
      await new Promise((resolve) => setTimeout(resolve, 15000));

      const [userSubscriptionPda] =
        anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("subscription"),
            subscriber.publicKey.toBuffer(),
            subscriptionPlanPda.toBuffer(),
          ],
          program.programId
        );

      let userSubs = await program.account.userSubscription.fetch(
        userSubscriptionPda
      );

      let task = taskKey(taskQueue, userSubs.nextTaskId)[0];

      await program.methods
        .cancelSubscription()
        .accountsPartial({
          subscriber: subscriber.publicKey,
          userSubscription: userSubscriptionPda,
          taskQueue,
          tokenProgram: TOKEN_PROGRAM_ID,
          task,
        })
        .signers([subscriber])
        .rpc();
    });
  });

  describe("close vault", () => {
    it("user can close his vault", async () => {
      await program.methods
        .closeVault()
        .accounts({
          subscriber: subscriber.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([subscriber])
        .rpc({ skipPreflight: true });
    });
  });
});
