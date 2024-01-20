import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  PublicKey,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
const BN = require("bn.js");
import {
  EscrowLayout,
  ESCROW_ACCOUNT_DATA_LAYOUT,
  getKeypair,
  getProgramId,
  getPublicKey,
  getTerms,
  getTokenBalance,
  logError,
} from "./utils";
import { establishConnection } from "./network";

const bob = async () => {
  const bobKeypair = getKeypair("bob");
  const bobUsdcTokenAccountPubkey = getPublicKey("bob_usdc");
  const escrowStateAccountPubkey = getPublicKey("escrow");
  const escrowProgramId = getProgramId();
  const terms = getTerms();

  const connection = await establishConnection();
  const escrowAccount = await connection.getAccountInfo(
    escrowStateAccountPubkey
  );
  
  if (escrowAccount === null) {
    logError("Could not find escrow at given address!");
    process.exit(1);
  }

  const encodedEscrowState = escrowAccount.data;
  const decodedEscrowLayout = ESCROW_ACCOUNT_DATA_LAYOUT.decode(
    encodedEscrowState
  ) as EscrowLayout;
  const escrowState = {
    escrowAccountPubkey: escrowStateAccountPubkey,
    isInitialized: !!decodedEscrowLayout.isInitialized,
    initializerAccountPubkey: new PublicKey(
      decodedEscrowLayout.initializerPubkey
    ),
    usdcTokenTempAccountPubkey: new PublicKey(
      decodedEscrowLayout.initializerTempTokenAccountPubkey
    ),
    expectedAmount: new BN(decodedEscrowLayout.expectedAmount, 10, "le"),
  };

  const PDA = await PublicKey.findProgramAddress(
    [Buffer.from("escrow")],
    escrowProgramId
  );

  const exchangeInstruction = new TransactionInstruction({
    programId: escrowProgramId,
    data: Buffer.from(
      Uint8Array.of(1, ...new BN(terms.bobExpectedAmount).toArray("le", 8))
    ),
    keys: [
      { pubkey: bobKeypair.publicKey, isSigner: true, isWritable: false },
      { pubkey: bobUsdcTokenAccountPubkey, isSigner: false, isWritable: true },
      {
        pubkey: escrowState.usdcTokenTempAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: escrowState.initializerAccountPubkey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: escrowStateAccountPubkey, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: PDA[0], isSigner: false, isWritable: false },
    ],
  });

  const bobUsdcbalance = await getTokenBalance(
    bobUsdcTokenAccountPubkey,
    connection
  );

  console.log("Sending Bob's transaction...");
  await connection.sendTransaction(
    new Transaction().add(exchangeInstruction),
    [bobKeypair],
    { skipPreflight: false, preflightCommitment: "confirmed" }
  );

  // sleep to allow time to update
  await new Promise((resolve) => setTimeout(resolve, 1000));

  if ((await connection.getAccountInfo(escrowStateAccountPubkey)) !== null) {
    logError("Escrow account has not been closed");
    process.exit(1);
  }

  if (
    (await connection.getAccountInfo(
      escrowState.usdcTokenTempAccountPubkey
    )) !== null
  ) {
    logError("Temporary Usdc token account has not been closed");
    process.exit(1);
  }

  const newBobUsdcbalance = await getTokenBalance(
    bobUsdcTokenAccountPubkey,
    connection
  );

  if (newBobUsdcbalance !== bobUsdcbalance + terms.bobExpectedAmount) {
    logError(
      `Bob's Usdc balance should be ${
        bobUsdcbalance + terms.bobExpectedAmount
      } but is ${newBobUsdcbalance}`
    );
    process.exit(1);
  }

  console.log(
    "✨Trade successfully executed. All temporary accounts closed✨\n"
  );
  console.table([
    {
      "Bob Token Account Usdc": newBobUsdcbalance,
    },
  ]);
  console.log("");
};

bob();
