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

  describe("cancel subscription", () => {
    it("user can cancel a subscription", async () => {
      let userSubscription = new anchor.web3.PublicKey(
        "9aWCGQRRZZnmcuCa6QikY6mqCuMMFkTFsokVXsdBdcex"
      );

      let userSubscriptionAccount =
        await program.account.userSubscription.fetch(
          userSubscription.toBase58()
        );

      let task = taskKey(taskQueue, userSubscriptionAccount.nextTaskId)[0];

      let s = await program.methods
        .cancelSubscription()
        .accountsPartial({
          subscriber: subscriber.publicKey,
          userSubscription,
          taskQueue,
          tokenProgram: TOKEN_PROGRAM_ID,
          task,
        })
        .signers([subscriber])
        .rpc();

      console.log(`https://explorer.solana.com/tx/${s}?cluster=devnet`);
    });
  });
});
