const fs = require("fs");
const {
  establishConnection,
  loadProgram,
  loadRegisters,
} = require("./network");

const programData = fs.readFileSync("./dist/program/solana_escrow.so");
const schema = require("../../configs/schema.json");
const { writePublicKey, getKeypair } = require("./utils");

(async () => {
  const connection = await establishConnection();
  const payer = await getKeypair("id");
  const program = await loadProgram(programData, payer, connection, true);
  const registers = await loadRegisters(schema, payer, program, connection);

  console.log("Deployment Info:");
  console.log("Payer:", payer.publicKey);
  console.log("Program:", program.address);
  writePublicKey(program.address, "program");

  registers.forEach(({ address, key }: any) => {
    console.log(`Register '${key}': ${address}`);
  });
})();

export {};
