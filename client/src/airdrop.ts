import { LAMPORTS_PER_SOL } from "@solana/web3.js";

import { getPublicKey } from "./utils";
import { establishConnection } from "./network";

const airdrop = async () => {
  const alicePublicKey = getPublicKey("alice");
  const bobPublicKey = getPublicKey("bob");
  const clientPublicKey = getPublicKey("id");

  const connection = await establishConnection();
  console.log("Requesting SOL for Alice...");
  // some networks like the local network provide an airdrop function (mainnet of course does not)
  await connection.requestAirdrop(alicePublicKey, LAMPORTS_PER_SOL * 10);
  console.log("Requesting SOL for Bob...");
  await connection.requestAirdrop(bobPublicKey, LAMPORTS_PER_SOL * 10);
  console.log("Requesting SOL for Client...");
  await connection.requestAirdrop(clientPublicKey, LAMPORTS_PER_SOL * 10);
};

airdrop();
