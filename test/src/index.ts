import { initializeKeypair } from "./initializeKeypair"
import * as web3 from "@solana/web3.js"
import * as borsh from "@coral-xyz/borsh"
import { CLUSTER, PROGRAM_ID } from "./constants"

async function main() {
  const connection = new web3.Connection(web3.clusterApiUrl(CLUSTER))
  const user = await initializeKeypair(connection)
  const programId = new web3.PublicKey(PROGRAM_ID);

  // serialize data
  const review = {
    title: "May I interrupt?",
    rating: 3,
    description: "May I interrupt? I do have something relevant to disclose."
  }
  const instructionSchema = borsh.struct([
    borsh.u8('command'),
    borsh.str('title'),
    borsh.u8('rating'),
    borsh.str('description')
  ])
  const buffer = Buffer.alloc(1000)
  instructionSchema.encode({
    ...review, command: 0
  }, buffer)
  const serializedBuffer = buffer.slice(0, instructionSchema.getSpan(buffer))

  // derive pda from combination of user key and title
  const [pda] = await web3.PublicKey.findProgramAddress(
    [user.publicKey.toBuffer(), new TextEncoder().encode(review.title)],
    programId
  )
  // make transaction
  const transaction = new web3.Transaction();
  const instruction = new web3.TransactionInstruction({
    keys: [
      {
        pubkey: user.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      }
    ],
    programId,
    data: serializedBuffer
  })
  transaction.add(instruction);
  const signature = await web3.sendAndConfirmTransaction(connection, transaction, [user]);
  console.log(`You can view your transaction on Solana Explorer at:\nhttps://explorer.solana.com/tx/${signature}?cluster=${CLUSTER}`)
}

main()
  .then(() => {
    console.log("Finished successfully")
    process.exit(0)
  })
  .catch((error) => {
    console.log(error)
    process.exit(1)
  })
