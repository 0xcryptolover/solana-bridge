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
console.log(feePayer.publicKey.toBase58());

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
    // const incognitoProxy = Keypair.generate();
    const incognitoProxy = Keypair.fromSecretKey(
        Uint8Array.from(Uint8Array.from([192,128,52,1,39,254,140,96,156,27,123,85,24,141,56,21,56,50,96,207,26,160,76,155,163,240,113,67,225,118,183,90,111,142,156,39,51,171,205,114,122,60,45,201,171,77,140,80,122,148,204,4,211,231,223,134,242,162,60,29,231,113,110,56]))
    );
    console.log(`incognito proxy: ${incognitoProxy.publicKey.toBase58()}`);

    const vaultAccount = Keypair.generate();
    // const beaconLength = 1315;
    // const lamportsExempt = await connection.getMinimumBalanceForRentExemption(beaconLength, 'confirmed');
    //
    // const transaction = new Transaction().add(
    //     SystemProgram.createAccount({
    //         fromPubkey: shieldMaker.publicKey,
    //         newAccountPubkey: incognitoProxy.publicKey,
    //         lamports: lamportsExempt,
    //         space: beaconLength,
    //         programId,
    //     }),
    // );
    // await sendAndConfirmTransaction(connection, transaction, [shieldMaker, incognitoProxy]);
    const [
        vaultTokenAuthority,
        bumpInit, // todo => store in incognito proxy
    ] = await PublicKey.findProgramAddress(
        [
            incognitoProxy.publicKey.toBuffer(),
        ],
        programId,
    );
    console.log("bump seed ", bumpInit);
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
    let vaultTokenAcc = new PublicKey("6dvNfGjtaErEefhUkDJtPhsxKxCxVDCMuVvyEdWsEgQu");

    console.log("=============== Init Beacon =================");

    // init beacon list address
    let beaconLength = 4;
    let beacon1 = [56,113,154,72,142,214,4,86,146,209,161,104,224,17,230,211,240,58,143,214,103,9,128,66,238,127,62,94,149,61,53,1,66,127,16,126,46,76,44,126,27,53,13,138,122,213,194,239,32,16,176,204,86,18,27,201,19,123,12,229,109,168,204,215];
    let beacon2 = [150,131,62,220,240,210,24,183,177,57,182,208,244,132,222,179,100,251,3,51,223,229,186,2,167,111,57,78,234,116,226,58,217,251,185,113,97,147,50,18,20,82,72,76,125,90,123,80,134,6,210,31,149,252,37,185,134,120,24,220,6,243,130,247];
    let beacon3 = [208,121,174,213,10,62,123,226,92,9,250,224,110,91,44,0,52,69,143,10,183,237,177,157,79,254,205,60,152,126,118,162,38,189,95,98,118,225,51,179,235,169,96,237,163,87,130,32,4,75,229,35,136,122,169,63,111,130,173,42,0,198,246,218];
    let beacon4 = [112,91,141,40,135,215,31,151,70,51,32,239,68,209,137,181,202,119,101,135,227,115,119,195,44,122,216,205,4,213,204,153,80,245,192,136,5,238,118,251,20,150,28,97,51,181,177,8,237,88,14,17,19,96,3,50,208,204,212,193,240,73,231,185];

    let pubkeyArray:number[] = Array.from(vaultAccount.publicKey.toBytes());
    const init_beacon_instruction = new TransactionInstruction({
        keys: [
            {pubkey: feePayer.publicKey, isSigner: true, isWritable: false},
            {pubkey: incognitoProxy.publicKey, isSigner: false, isWritable: true},
        ],
        programId,
        data: Buffer.from(
            Uint8Array.of(
                2,
                ...pubkeyArray,
                ...new BN(bumpInit).toArray("le", 1),
                ...new BN(beaconLength).toArray("le", 1),
                ...beacon1,
                ...beacon2,
                ...beacon3,
                ...beacon4,
            )
        ),
    });
    console.log("Beacon instruction length: ", init_beacon_instruction.data.length);
    // create transaction init beacon list
    const trans_init_beacon = await setPayerAndBlockhashTransaction(
        [init_beacon_instruction]
    );
    const signature_init_beacon = await signAndSendTransaction(trans_init_beacon, [feePayer]);
    const result_init_beacon = await connection.confirmTransaction(signature_init_beacon);
    console.log(`init beacon txhash: ${result_init_beacon}`);

    console.log("=============== Make shield request =================");

    let tokenAccount = await getAccount(connection, vaultTokenAcc);
    console.log(tokenAccount);
    // let incAddress = "";;
    var myBuffer:number[] = Array.from("12scKiKkL2ohYz6WF9zXGohgVqrJoRMtsbsJ8xhGNn1KNGhaEuW3SJEdPPTrhFxDJeG5wiyGr1BetJnok9Edrp4PhKxKAjF46UKTVAUTBMvD12ThrCqoDkr6WS7zSFoM9FvzP4xd6chZAtqfaTeq", (x) => x.charCodeAt(0));
    console.log("my buffer length ", myBuffer.length);
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
