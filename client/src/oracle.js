const {
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require('@solana/web3.js');
const { establishConnection, loadPayer, loadProgram } = require('./network');
const fs = require('fs');
const BN = require("bn.js");
const programData = fs.readFileSync('./dist/program/solana_escrow.so');
async function runOracle() {
  try {
    const connection = await establishConnection();
    const payer = await loadPayer(connection);

    const program = await loadProgram(programData, payer, connection);

    const intervalTime = 5000;
    setInterval(async () => {
      try {
        const instruction = new TransactionInstruction({
          programId: program.address,
          data: Buffer.from(
            Uint8Array.of(2, ...new BN(2).toArray("le", 8))
          ),
          keys: [],
        });

        const res = await sendAndConfirmTransaction(
          connection,
          new Transaction().add(instruction),
          [payer],
        );

        console.log(res);
      } catch (error) {
        console.error('Error in Oracle:', error);
      }
    }, intervalTime);

  } catch (error) {
    console.error('Oracle initialization error:', error);
  }
}

// Start the Oracle
runOracle();
