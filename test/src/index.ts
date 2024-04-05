import { initializeKeypair } from "./initializeKeypair"
import * as web3 from "@solana/web3.js"
import * as borsh from "@coral-xyz/borsh"
import { CLUSTER, PROGRAM_ID } from "./constants"
// import * as BN from "bignumber.js"
const BN = require('bn.js')
const programId = new web3.PublicKey(PROGRAM_ID);
class Wallet {
  connection: web3.Connection;
  user: web3.Keypair;

  constructor(connection: web3.Connection, user: web3.Keypair) {
    this.connection = connection;
    this.user = user;
  }
};

async function add_review(wallet: Wallet, title: string, rating: number, description: string) {
  // serialize data
  const instructionSchema = borsh.struct([
    borsh.u8('command'),
    borsh.str('title'),
    borsh.u8('rating'),
    borsh.str('description')
  ])
  const buffer = Buffer.alloc(1000)
  instructionSchema.encode({
    title, rating, description, command: 0
  }, buffer)
  const serializedBuffer = buffer.slice(0, instructionSchema.getSpan(buffer))

  // derive pda from combination of user key and title
  const [pda] = await web3.PublicKey.findProgramAddress(
    [wallet.user.publicKey.toBuffer(), new TextEncoder().encode(title)],
    programId
  )

  // derive counter pda with review pda and "comment" string
  const [counter_pda] = await web3.PublicKey.findProgramAddress(
    [pda.toBuffer(), Buffer.from("comment")], programId
  )
  // make transaction
  const transaction = new web3.Transaction();
  const instruction = new web3.TransactionInstruction({
    keys: [
      {
        pubkey: wallet.user.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: counter_pda,
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
  const signature = await web3.sendAndConfirmTransaction(wallet.connection, transaction, [wallet.user]);
  return signature
}

async function add_comment(wallet: Wallet, title: string, comment: string) {
  // serialize data
  const instructionSchema = borsh.struct([
    borsh.u8('command'),
    borsh.str('comment'),
  ])
  const buffer = Buffer.alloc(1000)
  instructionSchema.encode({
    comment, command: 2
  }, buffer)
  const serializedBuffer = buffer.slice(0, instructionSchema.getSpan(buffer))

  // derive pda from combination of user key and title
  const [pda_review] = web3.PublicKey.findProgramAddressSync(
    [wallet.user.publicKey.toBuffer(), new TextEncoder().encode(title)],
    programId
  )
  const [pda_counter] = web3.PublicKey.findProgramAddressSync(
    [pda_review.toBuffer(), new TextEncoder().encode("comment")],
    programId
  )
  const [pda_comment] = web3.PublicKey.findProgramAddressSync(
    [pda_review.toBuffer(), new BN([1]).toArrayLike(Buffer, "be", 8)],
    programId
  )
  console.log(`pda_review = ${pda_review}`)
  console.log(`pda_comment = ${pda_comment}`)
  // make transaction
  const transaction = new web3.Transaction();
  const instruction = new web3.TransactionInstruction({
    keys: [
      {
        pubkey: wallet.user.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: pda_review,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: pda_counter,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: pda_comment,
        isSigner: false,
        isWritable: true
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
  const signature = await web3.sendAndConfirmTransaction(wallet.connection, transaction, [wallet.user]);
  return signature
}

async function main() {
  const connection = new web3.Connection(web3.clusterApiUrl(CLUSTER))
  const user = await initializeKeypair(connection)
  const wallet = new Wallet(connection, user);
  // const signature = await add_review(
  //   wallet,
  //   "Can I make a comment?",
  //   3,
  //   "Can I make a comment? Mike was never involved in this."
  // )
  const signature = await add_comment(
    wallet,
    "Can I make a comment?",
    "Can I make a comment? I think Joan is already here.");
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
