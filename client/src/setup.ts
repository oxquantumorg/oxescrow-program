import { Connection, PublicKey, Signer } from "@solana/web3.js";

import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from "@solana/spl-token";
import {
  getKeypair,
  getPublicKey,
  getTokenBalance,
  writePublicKey,
} from "./utils";
import { establishConnection } from "./network";

const createMintUsdc = (
  connection: Connection,
  { publicKey, secretKey }: Signer
) => {
  return createMint(
    connection,
    {
      publicKey,
      secretKey,
    },
    publicKey,
    null,
    0
  );
};

const setupMint = async (
  name: string,
  connection: Connection,
  alicePublicKey: PublicKey,
  bobPublicKey: PublicKey,
  clientKeypair: Signer
) => {
  console.log(`Creating Mint ${name}...`);
  const mint = await createMintUsdc(connection, clientKeypair);
  console.log(mint);

  writePublicKey(mint, `mint_${name.toLowerCase()}`);

  console.log(`Creating Alice TokenAccount for ${name}...`);
  const aliceTokenAccount = await createAssociatedTokenAccount(
    connection,
    clientKeypair,
    mint,
    alicePublicKey
  );
  writePublicKey(aliceTokenAccount, `alice_${name.toLowerCase()}`);

  console.log(`Creating Bob TokenAccount for ${name}...`);
  const bobTokenAccount = await createAssociatedTokenAccount(
    connection,
    clientKeypair,
    mint,
    bobPublicKey
  );
  writePublicKey(bobTokenAccount, `bob_${name.toLowerCase()}`);

  return [mint, aliceTokenAccount, bobTokenAccount];
};

const setup = async () => {
  const aliceKeypair = getKeypair("alice");
  const alicePublicKey = getPublicKey("alice");
  const bobPublicKey = getPublicKey("bob");
  const clientKeypair = getKeypair("id");

  const connection = await establishConnection();

  const [mint, aliceTokenAccountForUsdc, bobTokenAccountForUsdc] =
    await setupMint(
      "Usdc",
      connection,
      alicePublicKey,
      bobPublicKey,
      clientKeypair
    );
  console.log("Sending 50Usdc to Alice's Usdc TokenAccount...");
  await mintTo(
    connection,
    aliceKeypair,
    mint,
    aliceTokenAccountForUsdc,
    clientKeypair,
    50
  );

  console.log("✨Setup complete✨\n");
  console.table([
    {
      "Alice Token Account Usdc": await getTokenBalance(
        aliceTokenAccountForUsdc,
        connection
      ),
      "Bob Token Account Usdc": await getTokenBalance(
        bobTokenAccountForUsdc,
        connection
      ),
    },
  ]);
  console.log("");
};

setup();
