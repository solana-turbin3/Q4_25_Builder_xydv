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

  const tuktukConfig = tuktukConfigKey()[0];

  const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new anchor.web3.PublicKey(
    "BPFLoaderUpgradeab1e11111111111111111111111"
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

  it("Is initialized!", async () => {
    const tx = await program.account.globalState.all();

    console.log("Your transaction signature", tx);
  });
});
