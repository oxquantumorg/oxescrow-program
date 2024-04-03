import {
  createInitializeAccountInstruction,
  createTransferInstruction,
  AccountLayout,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { getProgramId } from "./utils";
import { sendAndConfirmTransaction } from "@solana/web3.js";

const {
  Account,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
} = require("@solana/web3.js");
const BN = require("bn.js");
const {
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
  const bobKeypair = getKeypair("bob");
  const initializerAccount = getKeypair("alice");
  const callerAcc = getKeypair("id");

  const tempUsdcTokenAccountKeypair = new Account();

  const tempTokenAccountIX = SystemProgram.createAccount({
    programId: TOKEN_PROGRAM_ID,
    space: AccountLayout.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span
    ),
    fromPubkey: callerAcc.publicKey,
    newAccountPubkey: tempUsdcTokenAccountKeypair.publicKey,
  });
  const initTempAccountIX = createInitializeAccountInstruction(
    tempUsdcTokenAccountKeypair.publicKey,
    usdcTokenMintPubkey,
    callerAcc.publicKey
  );
  const transferUsdcTokensToTempAccIX = createTransferInstruction(
    aliceUsdcTokenAccountPubkey,
    tempUsdcTokenAccountKeypair.publicKey,
    initializerAccount.publicKey,
    terms.transferAmount
  );

  const escrowKeypair = new Account();
  const createEscrowAccountIX = SystemProgram.createAccount({
    space: ESCROW_ACCOUNT_DATA_LAYOUT.span,
    lamports: await connection.getMinimumBalanceForRentExemption(
      ESCROW_ACCOUNT_DATA_LAYOUT.span
    ),
    fromPubkey: callerAcc.publicKey,
    newAccountPubkey: escrowKeypair.publicKey,
    programId: escrowProgramId,
  });

  const initEscrowIUsdc = new TransactionInstruction({
    programId: escrowProgramId,
    keys: [
      {
        pubkey: initializerAccount.publicKey,
        isSigner: false,
        isWritable: false,
      },
      // { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      {
        pubkey: callerAcc.publicKey,
        isSigner: true,
        isWritable: false,
      },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },

    ],
    data: Buffer.from(
      Uint8Array.of(0, ...new BN(terms.transferAmount).toArray("le", 1))
    ),
  });

  const tx = new Transaction().add(
    tempTokenAccountIX,
    initTempAccountIX,
    transferUsdcTokensToTempAccIX,
    createEscrowAccountIX,
    initEscrowIUsdc
  );

  const res = await sendAndConfirmTransaction(connection, tx, [
    callerAcc,
    initializerAccount,
    tempUsdcTokenAccountKeypair,
    escrowKeypair,
  ]);
  console.log("Escrow Account:", escrowKeypair.publicKey);
  console.log("Transaction Hash:", res);

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
  console.log("Escrow state:", decodedEscrowState);

  if (!decodedEscrowState.isInitialized) {
    logError("Escrow state initialization flag has not been set");
    process.exit(1);
  } else if (
    !new PublicKey(decodedEscrowState.initializerPubkey).equals(
      initializerAccount.publicKey
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
    `✨Escrow successfully initialized. Alice is offering ${terms.transferAmount}Usdc\n`
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
