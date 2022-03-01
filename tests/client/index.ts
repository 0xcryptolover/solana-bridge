import { clusterApiUrl, Connection, Keypair, Transaction, SystemProgram, PublicKey, sendAndConfirmTransaction, TransactionInstruction } from "@solana/web3.js";
import {
    createInitializeMintInstruction,
    TOKEN_PROGRAM_ID,
    MINT_SIZE,
    getMinimumBalanceForRentExemptMint,
    createMint,
    createAssociatedTokenAccount,
    getAssociatedTokenAddress,
    createMintToCheckedInstruction,
    createAssociatedTokenAccountInstruction,
    mintToChecked,
    getAccount,
} from "@solana/spl-token";
import * as bs58 from "bs58";
import { deserialize, serialize } from "borsh";
import BN = require("bn.js");
// vault program
const programId = new PublicKey(
    "BKGhwbiTHdUxcuWzZtDWyioRBieDEXTtgEk8u1zskZnk"
);

// connection
const connection = new Connection(clusterApiUrl("devnet"), "confirmed");

// 5YNmS1R9nNSCDzb5a7mMJ1dwK9uHeAAF4CmPEwKgVWr8
const feePayer = Keypair.fromSecretKey(
    bs58.decode("588FU4PktJWfGfxtzpAAXywSNt74AvtroVzGfKkVN1LwRuvHwKGr851uH8czM5qm4iqLbs1kKoMKtMJG4ATR7Ld2")
);

// G2FAbFQPFa5qKXCetoFZQEvF9BVvCKbvUZvodpVidnoY
const shieldMaker = Keypair.fromSecretKey(
    Uint8Array.from(Uint8Array.from([56,52,143,70,102,247,217,158,213,127,195,28,52,49,229,216,186,136,63,94,185,108,216,64,35,120,204,184,221,151,114,120,92,163,172,46,113,242,87,204,236,137,51,132,55,203,117,88,87,243,21,194,162,119,17,200,227,147,2,222,181,12,77,224]))
);

(async () => {

    // 1) use build-in function
    // let mintPubkey = await createMint(
    //     connection, // conneciton
    //     feePayer, // fee payer
    //     alice.publicKey, // mint authority
    //     alice.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
    //     8 // decimals
    // );
    // console.log(`mint: ${mintPubkey.toBase58()}`);
    const mintPubkey = new PublicKey("EHheP6Wfyz65ve258TYQcfBHAAY4LsErnmXZozrgfvGr");

    // 1) use build-in function
    // {
    //     let ata = await createAssociatedTokenAccount(
    //         connection, // connection
    //         feePayer, // fee payer
    //         mintPubkey, // mint
    //         alice.publicKey // owner,
    //     );
    //     console.log(`ATA: ${ata.toBase58()}`);
    // }

    // or

    // 2) composed by yourself
    // {
    //     // calculate ATA
    //     let ata = await getAssociatedTokenAddress(
    //         mintPubkey, // mint
    //         alice.publicKey // owner
    //     );
    //     console.log(`ATA: ${ata.toBase58()}`);
    //
    //     // if your wallet is off-curve, you should use
    //     // let ata = await getAssociatedTokenAddress(
    //     //   mintPubkey, // mint
    //     //   alice.publicKey // owner
    //     //   true, // allowOwnerOffCurve
    //     // );
    //
    //     let tx = new Transaction().add(
    //         createAssociatedTokenAccountInstruction(
    //             feePayer.publicKey, // payer
    //             ata, // ata
    //             alice.publicKey, // owner
    //             mintPubkey // mint
    //         )
    //     );
    //     console.log(`txhash: ${await connection.sendTransaction(tx, [feePayer])}`);
    // }
    const shieldMakerAccount = new PublicKey("5397KrEBCuEhdTjWF5B9xjVzGJR6MyxXLP3srbrWo2gD");
    const shield_amount = 10000;
    // mint token to shield maker token account
    {
        let txhash = await mintToChecked(
            connection, // connection
            feePayer, // fee payer
            mintPubkey, // mint
            shieldMakerAccount, // receiver (sholud be a token account)
            shieldMaker, // mint authority
            100e8, // amount. if your decimals is 8, you mint 10^8 for 1 token.
            8 // decimals
        );
        console.log(`txhash: ${txhash}`);
    }

    //     // if your wallet is off-curve, you should use
    //     // let ata = await getAssociatedTokenAddress(
    //     //   mintPubkey, // mint
    //     //   alice.publicKey // owner
    //     //   true, // allowOwnerOffCurve
    //     // );
    //
    //     let tx = new Transaction().add(
    //         createAssociatedTokenAccountInstruction(
    //             feePayer.publicKey, // payer
    //             ata, // ata
    //             alice.publicKey, // owner
    //             mintPubkey // mint
    //         )
    //     );
    const incognitoProxy = Keypair.generate();
    console.log(`incognito proxy: ${incognitoProxy.publicKey.toBase58()}`);

    const transaction = new Transaction().add(
        SystemProgram.createAccount({
            fromPubkey: shieldMaker.publicKey,
            newAccountPubkey: incognitoProxy.publicKey,
            lamports: 5000,
            space: (35 + 64 * 5),
            programId,
        }),
    );
    await sendAndConfirmTransaction(connection, transaction, [shieldMaker, incognitoProxy]);
    const [
        vaultTokenAuthority,
        bumpInit, // todo => store in incognito proxy
    ] = await PublicKey.findProgramAddress(
        [
            incognitoProxy.publicKey.toBuffer(),
        ],
        programId,
    );
    // let vaultTokenAcc = await getAssociatedTokenAddress(
    //     mintPubkey, // mint
    //     vaultTokenAuthority, // owner
    //   true, // allowOwnerOffCurve
    // );
    // console.log(`token account owned by vault : ${vaultTokenAcc.toBase58()}`);
    //
    // let tx = new Transaction().add(
    //     createAssociatedTokenAccountInstruction(
    //         feePayer.publicKey, // payer
    //         vaultTokenAcc, // ata
    //         vaultTokenAuthority, // owner
    //         mintPubkey // mint
    //     )
    // );
    // console.log(`txhash: ${await connection.sendTransaction(tx, [feePayer])}`);
    let vaultTokenAcc = new PublicKey("CKsZSXcrCxye6rjTFZqUBpfxqrjhd9ogJtHZLsQ5iQaF");
    let tokenAccount = await getAccount(connection, vaultTokenAcc);
    console.log(tokenAccount);
    // let incAddress = "";;
    var myBuffer:number[] = Array.from("12scKiKkL2ohYz6WF9zXGohgVqrJoRMtsbsJ8xhGNn1KNGhaEuW3SJEdPPTrhFxDJeG5wiyGr1BetJnok9Edrp4PhKxKAjF46UKTVAUTBMvD12ThrCqoDkr6WS7zSFoM9FvzP4xd6chZAtqfaTeq", (x) => x.charCodeAt(0));
    console.log("my buffer length ", myBuffer.length);
    // var buffer = new Buffer(incAddress, 'utf16le');
    // for (var i = 0; i < buffer.length; i++) {
    //     myBuffer.push(buffer[i]);
    // }
    console.log(myBuffer);
    // shield request
    // let shielRequestInfo = new ShieldDetails({
    //     amount: shield_amount,
    //     inc_address: buffer,
    // })
    const instruction = new TransactionInstruction({
        keys: [
            {pubkey: shieldMakerAccount, isSigner: false, isWritable: true},
            {pubkey: vaultTokenAcc, isSigner: false, isWritable: true},
            {pubkey: incognitoProxy.publicKey, isSigner: false, isWritable: false},
            {pubkey: shieldMaker.publicKey, isSigner: true, isWritable: false},
            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false},
        ],
        programId,
        data: Buffer.from(
            Uint8Array.of(0, ...new BN(shield_amount).toArray("le", 8), ...myBuffer)
        ),
    });

    const trans = await setPayerAndBlockhashTransaction(
        [instruction]
    );
    const signature = await signAndSendTransaction(trans, [feePayer, shieldMaker]);
    const result = await connection.confirmTransaction(signature);
    console.log("end sendMessage", result);

})();


export async function setPayerAndBlockhashTransaction(instructions: any) {
    const transaction = new Transaction();
    instructions.forEach((element: any) => {
        transaction.add(element);
    });
    transaction.feePayer = feePayer.publicKey;
    let hash = await connection.getLatestBlockhash();
    transaction.recentBlockhash = hash.blockhash;
    return transaction;
}

async function signAndSendTransaction(transaction: any, listSigners : any) {
    try {
        console.log("start signAndSendTransaction");
        let txResult = await sendAndConfirmTransaction(
            connection,
            transaction,
            listSigners,
        );
        console.log("end signAndSendTransaction");
        return txResult;
    } catch (err) {
        console.log("signAndSendTransaction error", err);
        throw err;
    }
}

class ShieldDetails {
    constructor(properties: any) {
        Object.keys(properties).forEach((key) => {
            console.log({key});
            // this[key] = properties[key] as any;
        });
    }
    static schema = new Map([[ShieldDetails,
        {
            kind: 'struct',
            fields: [
                ['amount', 'u64'],
                ['inc_address', [148]]]
        }]]);
}

// export async function sayHello(): Promise<void> {
//     console.log('Saying hello to', greetedPubkey.toBase58());
//     const instruction = new TransactionInstruction({
//         keys: [{pubkey: greetedPubkey, isSigner: false, isWritable: true}],
//         programId,
//         data: Buffer.alloc(0), // All instructions are hellos
//     });
//     await sendAndConfirmTransaction(
//         connection,
//         new Transaction().add(instruction),
//         [payer],
//     );
// }