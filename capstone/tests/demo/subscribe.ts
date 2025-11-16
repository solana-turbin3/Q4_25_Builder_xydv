import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Capstone } from "../../target/types/capstone";
import {
  init as initTuktuk,
  nextAvailableTaskIds,
  taskKey,
} from "@helium/tuktuk-sdk";
import { Tuktuk } from "@helium/tuktuk-idls/lib/types/tuktuk";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("capstone", () => {
  const provider = anchor.AnchorProvider.local(
    "https://devnet.helius-rpc.com/?api-key=b9e05c06-0429-46ac-b3b4-91d56edb88ba"
  );

  anchor.setProvider(provider);

  const program = anchor.workspace.capstone as Program<Capstone>;
  const signer = provider.wallet.publicKey;

  // so that we can close this after demo
  const subscriber = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from([
      116, 77, 230, 125, 125, 33, 29, 71, 234, 94, 90, 190, 183, 3, 108, 4, 89,
      35, 161, 71, 61, 161, 94, 130, 94, 4, 55, 203, 13, 111, 38, 201, 13, 139,
      57, 224, 89, 146, 109, 147, 204, 16, 58, 221, 153, 15, 71, 18, 143, 217,
      224, 86, 170, 185, 205, 196, 243, 135, 124, 115, 206, 56, 0, 196,
    ])
  );

  console.log("subscriber: ", subscriber.publicKey.toBase58());

  let taskQueue: anchor.web3.PublicKey;
  let tuktukProgram: Program<Tuktuk>;

  const [globalStatePda, globalStateBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("global")],
      program.programId
    );

  const USDC_MINT = new anchor.web3.PublicKey(
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" // for devnet
  );

  before(async () => {
    const globalState = await program.account.globalState.fetchNullable(
      globalStatePda
    );

    // @ts-ignore
    tuktukProgram = await initTuktuk(provider);
    taskQueue = globalState.taskQueue;
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

      const subscriptionPlan = new anchor.web3.PublicKey(
        "6dFt2ND45VS5xe2CawyfQQ4ZUKoGntfw1avmxjpsHs3w"
      );

      await program.methods
        .subscribe()
        .accountsPartial({
          subscriber: subscriber.publicKey,
          subscriptionPlan,
          mint: USDC_MINT,
          task: taskKey(taskQueue, nextTask)[0],
          globalState: globalStatePda,
          taskQueue,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([subscriber])
        .rpc();
    });
  });
});
