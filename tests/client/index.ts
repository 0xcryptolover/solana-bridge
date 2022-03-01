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
    console.log("shield maker ", shieldMakerAccount.toBytes());
    console.log("token id ", mintPubkey.toBytes());
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
    let beacon1 = [64,206,253,84,56,206,63,162,157,152,148,80,198,23,66,245,43,1,207,238,9,144,161,139,131,44,146,136,74,242,22,220,187,130,145,153,93,114,117,199,108,190,233,244,53,240,247,48,207,19,94,245,14,171,207,124,157,177,173,139,253,237,36,168];
    let beacon2 = [175,109,126,18,52,108,137,78,38,252,216,214,224,214,44,187,2,67,70,204,196,78,155,224,72,126,124,128,134,165,210,158,138,93,62,90,76,225,186,39,215,204,170,10,127,99,86,220,107,251,34,58,235,236,69,189,235,226,57,208,106,210,28,22];
    let beacon3 = [122,69,179,100,37,117,17,36,0,4,211,125,150,102,106,180,218,127,238,200,104,84,250,183,23,31,209,229,22,117,248,73,56,120,112,2,188,187,152,44,70,228,25,160,250,255,40,216,180,239,183,235,175,79,66,41,119,82,195,70,103,102,135,73];
    let beacon4 = [24,171,11,173,118,80,213,52,20,186,77,213,182,249,188,70,15,37,228,129,102,45,183,139,139,174,147,32,130,179,168,171,36,79,30,237,44,11,200,229,108,224,117,224,206,11,62,235,127,101,194,116,209,213,122,41,77,229,19,60,199,168,81,25];

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
    // todo: query beacon state before init
    // create transaction init beacon list
    // const trans_init_beacon = await setPayerAndBlockhashTransaction(
    //     [init_beacon_instruction]
    // );
    // const signature_init_beacon = await signAndSendTransaction(trans_init_beacon, [feePayer]);
    // const result_init_beacon = await connection.confirmTransaction(signature_init_beacon);
    // console.log(`init beacon txhash: ${result_init_beacon}`);

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

    console.log("=============== Make unshield request =================");
    // create data
    let inst = [155,1,59,251,20,8,169,41,14,126,217,202,208,133,245,194,62,249,144,33,114,189,69,8,203,236,33,202,139,252,22,182,84,166,197,111,46,11,236,137,61,81,37,248,63,18,210,220,135,170,35,25,197,242,222,184,44,97,54,230,121,239,31,201,122,75,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,232,212,165,16,0,175,50,183,8,230,168,203,6,123,213,253,17,144,50,145,211,56,177,99,211,190,107,72,141,144,169,52,211,143,77,245,86,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let height = 1;
    // =====
    let inst_paths = [[230,184,120,45,145,71,239,155,84,73,101,255,16,237,227,249,93,245,68,65,52,188,45,171,214,48,218,58,142,126,159,241],
        [50,19,205,111,207,123,115,108,252,197,54,235,131,223,254,252,171,24,167,244,113,233,90,51,224,58,124,77,61,171,131,135],
        [210,186,109,37,156,189,54,252,198,35,128,107,1,64,214,65,247,222,71,39,45,204,183,120,231,117,31,173,134,29,208,54],
        [50,36,225,183,195,238,157,68,33,143,0,255,168,100,62,38,48,211,141,19,176,16,50,39,182,180,229,59,184,184,220,14]
    ];
    // =====
    let inst_path_is_lefts = [true, true, true, false]
    // =====
    let inst_root = [4,149,252,107,43,40,15,174,250,71,77,164,169,166,235,185,18,186,5,206,47,73,152,113,20,51,33,134,193,116,17,37];
    let blkdata = [132,84,236,128,76,197,24,200,64,97,202,40,182,30,81,85,26,44,174,75,120,239,182,206,107,66,245,76,172,183,110,14];
    // ====
    let index = [1, 2, 3, 4]
    let signatures = [
        [125,90,168,210,201,85,7,195,187,77,26,108,52,116,24,77,42,170,70,159,117,88,183,174,207,127,11,71,243,32,147,202,125,119,211,3,131,214,202,149,225,121,67,153,110,131,130,67,219,216,21,191,178,223,117,201,74,52,173,238,23,79,168,78,27],
        [88,12,238,123,254,29,46,56,104,159,162,58,3,63,165,67,121,166,58,94,49,80,206,239,78,33,230,205,162,17,104,220,116,71,3,177,62,112,136,54,196,34,106,245,178,6,75,201,112,100,95,184,59,149,198,225,98,210,57,12,205,66,6,162,27],
        [81,208,206,204,130,189,88,203,51,226,129,22,183,70,71,114,179,4,245,213,243,41,207,103,3,110,115,88,3,148,108,231,119,102,130,250,102,74,201,11,177,182,108,225,126,250,108,197,153,245,4,167,247,179,10,43,62,121,41,190,206,6,55,111,27],
        [240,52,229,173,211,50,154,27,201,224,129,8,41,131,145,99,24,235,220,20,226,53,181,63,26,159,65,176,237,83,222,194,51,190,201,48,17,1,105,216,2,42,20,177,104,117,135,156,92,92,7,178,204,5,185,45,223,49,36,18,48,112,3,178,27]
    ];
    // =====



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
