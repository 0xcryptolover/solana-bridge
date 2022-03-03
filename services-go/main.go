package main

import (
	"context"
	"fmt"
	"github.com/davecgh/go-spew/spew"
	"github.com/gagliardetto/solana-go/text"
	"os"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/rpc"
	confirm "github.com/gagliardetto/solana-go/rpc/sendAndConfirmTransaction"
	"github.com/gagliardetto/solana-go/rpc/ws"
	"strings"
)

const SHIELD = "Shield"
const UNSHIELD = "Unshield"
const INCOGNITO_PROXY = "8WUP1RGTDTZGYBjkHQfjnwMbnnk25hnE6Du7vFpaq1QK"
const PROGRAM_ID = "BKGhwbiTHdUxcuWzZtDWyioRBieDEXTtgEk8u1zskZnk"

func main() {
	// init vars
	// Create a new WS client (used for confirming transactions)
	wsClient, err := ws.Connect(context.Background(), rpc.DevNet_WS)
	if err != nil {
		panic(err)
	}

	program := solana.MustPublicKeyFromBase58(PROGRAM_ID)
	incognitoProxy := solana.MustPublicKeyFromBase58(INCOGNITO_PROXY)
	feePayer, err := solana.PrivateKeyFromBase58("588FU4PktJWfGfxtzpAAXywSNt74AvtroVzGfKkVN1LwRuvHwKGr851uH8czM5qm4iqLbs1kKoMKtMJG4ATR7Ld2")
	if err != nil {
		panic(err)
	}
	shieldMaker, err := solana.PrivateKeyFromBase58("28BD5MCpihGHD3zUfPv4VcBizis9zxFdaj9fGJmiyyLmezT94FRd9XiiLjz5gasgyX3TmH1BU4scdVE6gzDFzrc7")
	if err != nil {
		panic(err)
	}
	// Create a new RPC client:
	rpcClient := rpc.New(rpc.DevNet_RPC)

	client, err := ws.Connect(context.Background(), rpc.DevNet_WS)
	if err != nil {
		panic(err)
	}

	// test shield tx
	recent, err := rpcClient.GetRecentBlockhash(context.TODO(), rpc.CommitmentFinalized)
	if err != nil {
		panic(err)
	}

	shieldMakerTokenAccount := solana.MustPublicKeyFromBase58("5397KrEBCuEhdTjWF5B9xjVzGJR6MyxXLP3srbrWo2gD")
	vaultTokenAcc := solana.MustPublicKeyFromBase58("6dvNfGjtaErEefhUkDJtPhsxKxCxVDCMuVvyEdWsEgQu")
	accounts := solana.AccountMetaSlice{}
	err = accounts.SetAccounts(
		[]*solana.AccountMeta{
			solana.NewAccountMeta(shieldMakerTokenAccount, true, false),
			solana.NewAccountMeta(vaultTokenAcc, true, false),
			solana.NewAccountMeta(incognitoProxy, false, false),
			solana.NewAccountMeta(shieldMaker.PublicKey(), false, true),
			solana.NewAccountMeta(solana.TokenProgramID, false, false),
		},
	)
	if err != nil {
		panic(err)
	}
	signers := []solana.PrivateKey{
		feePayer,
		shieldMaker,
	}
	//accountTo := solana.MustPublicKeyFromBase58("6dvNfGjtaErEefhUkDJtPhsxKxCxVDCMuVvyEdWsEgQu")
	shieldInstruction := solana.NewInstruction(
			program,
			accounts,
			[]byte{
				0,   0,  16, 165, 212, 232,   0,   0,   0,  49,  50, 115,
				99,  75, 105,  75, 107,  76,  50, 111, 104,  89, 122,  54,
				87,  70,  57, 122,  88,  71, 111, 104, 103,  86, 113, 114,
				74, 111,  82,  77, 116, 115,  98, 115,  74,  56, 120, 104,
				71,  78, 110,  49,  75,  78,  71, 104,  97,  69, 117,  87,
				51,  83,  74,  69, 100,  80,  80,  84, 114, 104,  70, 120,
				68,  74, 101,  71,  53, 119, 105, 121,  71, 114,  49,  66,
				101, 116,  74, 110, 111, 107,  57,  69, 100, 114, 112,  52,
				80, 104,  75, 120,  75,  65, 106,  70,  52,  54,  85,  75,
				84,  86,  65,  85,  84,  66,  77, 118,  68,  49,  50,  84,
				104, 114,  67, 113, 111,  68, 107, 114,  54,  87,  83,  55,
				122,  83,  70, 111,  77,  57,  70, 118, 122,  80,  52, 120,
				100,  54,  99, 104,  90,  65, 116, 113, 102,  97,  84, 101,
				113,
			},
		)

	//amount := uint64(1000)
	tx, err := solana.NewTransaction(
		[]solana.Instruction{
			shieldInstruction,
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(feePayer.PublicKey()),
	)
	if err != nil {
		panic(err)
	}

	_, err = tx.Sign(func(key solana.PublicKey) *solana.PrivateKey {
		for _, candidate := range signers {
			if candidate.PublicKey().Equals(key) {
				return &candidate
			}
		}
		return nil
	})
	if err != nil {
		panic(fmt.Errorf("unable to sign transaction: %w", err))
	}
	spew.Dump(tx)
	// Pretty print the transaction:
	tx.EncodeTree(text.NewTreeEncoder(os.Stdout, "Shield "))

	// Send transaction, and wait for confirmation:
	sig, err := confirm.SendAndConfirmTransaction(
		context.Background(),
		rpcClient,
		wsClient,
		tx,
	)
	if err != nil {
		panic(err)
	}
	spew.Dump(sig)

	{
		// Subscribe to log events that mention the provided pubkey:
		sub, err := client.LogsSubscribeMentions(
			program,
			rpc.CommitmentRecent,
		)
		if err != nil {
			panic(err)
		}
		defer sub.Unsubscribe()

		for {
			got, err := sub.Recv()
			if err != nil {
				panic(err)
			}
			// dump to struct { signature , error, value }
			spew.Dump(got)
			processShield(got)
		}
	}
}

func processShield(logs *ws.LogResult) {
	if logs.Value.Err != nil {
		fmt.Printf("the transaction failed %v \n", logs.Value.Err)
		return
	}

	if len(logs.Value.Logs) < 7 {
		fmt.Printf("invalid shield logs, length must greate than 7 %v \n", logs.Value.Err)
		return
	}
	// todo: check signature and store if new
	//logs.Value.Signature

	shieldLogs := logs.Value.Logs
	// check shield instruction
	if !strings.Contains(shieldLogs[1], SHIELD) {
		fmt.Printf("invalid instruction %s\n", shieldLogs[1])
		return
	}

	shieldInfoSplit := strings.Split(shieldLogs[6], ":")
	if len(shieldInfoSplit) < 3 {
		fmt.Printf("invalid shield logs %+v\n", logs)
		return
	}

	shieldInfo := strings.Split(shieldInfoSplit[2], ",")
	if len(shieldInfo) < 4 {
		fmt.Printf("invalid shield info %v\n", shieldInfo)
		return
	}

	incognitoProxy := shieldInfo[0]
	if incognitoProxy != INCOGNITO_PROXY {
		fmt.Printf("invalid incognito proxy %v \n", incognitoProxy)
		return
	}
	incAddress := shieldInfo[1]
	tokenID := shieldInfo[2]
	amount := shieldInfo[3]

	fmt.Printf("shield with inc address %s token id %s and amount %s \n", incAddress, tokenID, amount)
}