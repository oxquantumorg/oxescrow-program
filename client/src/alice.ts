import { createInitializeAccountInstruction, createTransferInstruction } from "@solana/spl-token";
import { getProgramId } from "./utils";

const { AccountLayout, Token, TOKEN_PROGRAM_ID } = require("@solana/spl-token");
const {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
} = require("@solana/web3.js");
const BN = require("bn.js");
const fs = require("fs");
const {
  EscrowLayout,
  getKeypair,
  ESCROW_ACCOUNT_DATA_LAYOUT,
  getPublicKey,
  getTerms,
  getTokenBalance,
  logError,
  writePublicKey,
} = require("./utils");
const { establishConnection } = require("./network");

const alice = async () => {
  const connection = await establishConnection();
  const escrowProgramId = getProgramId();
  const terms = getTerms();

  const aliceUsdcTokenAccountPubkey = getPublicKey("alice_usdc");
  const usdcTokenMintPubkey = getPublicKey("mint_usdc");
  const aliceKeypair = getKeypair("alice");
  
  const tempUsdcTokenAccountKeypair = new Keypair();

  const createTempTokenAccountIUsdc = SystemProgram.createAccount({
    programId: TOKEN_PROGRAM_ID,
    space: AccountLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span
    ),
    fromPubkey: aliceKeypair.publicKey,
    newAccountPubkey: tempUsdcTokenAccountKeypair.publicKey,
  });
  const initTempAccountIUsdc = createInitializeAccountInstruction(
    TOKEN_PROGRAM_ID,
    usdcTokenMintPubkey,
    tempUsdcTokenAccountKeypair.publicKey,
    aliceKeypair.publicKey
  );
  const transferUsdcTokensToTempAccIUsdc = createTransferInstruction(
    aliceUsdcTokenAccountPubkey,
    tempUsdcTokenAccountKeypair.publicKey,
    aliceUsdcTokenAccountPubkey,
    terms.bobExpectedAmount,
    undefined,
    TOKEN_PROGRAM_ID
  );

  const escrowKeypair = new Keypair();
  const createEscrowAccountIUsdc = SystemProgram.createAccount({
    space: ESCROW_ACCOUNT_DATA_LAYOUT.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      ESCROW_ACCOUNT_DATA_LAYOUT.span
    ),
    fromPubkey: aliceKeypair.publicKey,
    newAccountPubkey: escrowKeypair.publicKey,
    programId: escrowProgramId,
  });
  
  const initEscrowIUsdc = new TransactionInstruction({
    programId: escrowProgramId,
    keys: [
      { pubkey: aliceKeypair.publicKey, isSigner: true, isWritable: false },
      {
        pubkey: tempUsdcTokenAccountKeypair.publicKey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: escrowKeypair.publicKey, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(
      Uint8Array.of(0, ...new BN(terms.aliceExpectedAmount).toArray("le", 8))
    ),
  });

  const tx = new Transaction().add(
    createTempTokenAccountIUsdc,
    initTempAccountIUsdc,
    transferUsdcTokensToTempAccIUsdc,
    createEscrowAccountIUsdc,
    initEscrowIUsdc
  );

  const blockhash = (await connection.getLatestBlockhash('finalized')).blockhash;
  tx.recentBlockhash = blockhash;
  await tx.sign(aliceKeypair);

  console.log("Sending Alice's transaction...");
  await connection.sendTransaction(
    tx,
    [aliceKeypair, tempUsdcTokenAccountKeypair, escrowKeypair],
    { skipPreflight: false, preflightCommitment: "confirmed" }
  );

  // sleep to allow time to update
  await new Promise((resolve) => setTimeout(resolve, 1000));

  const escrowAccount = await connection.getAccountInfo(
    escrowKeypair.publicKey
  );

  if (escrowAccount === null || escrowAccount.data.length === 0) {
    logError("Escrow state account has not been initialized properly");
    process.exit(1);
  }

  const encodedEscrowState = escrowAccount.data;
  const decodedEscrowState =
    ESCROW_ACCOUNT_DATA_LAYOUT.decode(encodedEscrowState);

  if (!decodedEscrowState.isInitialized) {
    logError("Escrow state initialization flag has not been set");
    process.exit(1);
  } else if (
    !new PublicKey(decodedEscrowState.initializerPubkey).equals(
      aliceKeypair.publicKey
    )
  ) {
    logError(
      "InitializerPubkey has not been set correctly / not been set to Alice's public key"
    );
    process.exit(1);
  } else if (
    !new PublicKey(decodedEscrowState.initializerTempTokenAccountPubkey).equals(
      tempUsdcTokenAccountKeypair.publicKey
    )
  ) {
    logError(
      "initializerTempTokenAccountPubkey has not been set correctly / not been set to temp Usdc token account public key"
    );
    process.exit(1);
  }
  console.log(
    `✨Escrow successfully initialized. Alice is offering ${terms.bobExpectedAmount}Usdc\n`
  );
  writePublicKey(escrowKeypair.publicKey, "escrow");
  console.table([
    {
      "Alice Token Account Usdc": await getTokenBalance(
        aliceUsdcTokenAccountPubkey,
        connection
      ),
      "Bob Token Account Usdc": await getTokenBalance(
        getPublicKey("bob_usdc"),
        connection
      ),
      "Temporary Token Account Usdc": await getTokenBalance(
        tempUsdcTokenAccountKeypair.publicKey,
        connection
      ),
    },
  ]);

  console.log("");
};

alice();
