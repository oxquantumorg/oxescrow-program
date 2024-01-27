import { mintTo } from "@solana/spl-token";
import { getKeypair, getPublicKey, getTokenBalance } from "./utils";
import { establishConnection } from "./network";

const setup = async () => {
  const aliceKeypair = getKeypair("alice");
  const clientKeypair = getKeypair("id");
  const mint = getPublicKey("mint_usdc");
  const aliceTokenAccountForUsdc = getPublicKey("alice_usdc");
  const bobTokenAccountForUsdc = getPublicKey("bob_usdc");

  const connection = await establishConnection();
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
