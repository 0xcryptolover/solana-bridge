package main

import (
	"context"
	"fmt"
	"github.com/davecgh/go-spew/spew"
	"github.com/gagliardetto/solana-go"
	associatedtokenaccount "github.com/gagliardetto/solana-go/programs/associated-token-account"
	"github.com/gagliardetto/solana-go/programs/system"
	"github.com/gagliardetto/solana-go/rpc"
	"github.com/gagliardetto/solana-go/rpc/ws"
	"github.com/thachtb/solana-bridge/services-go/Shield"
	unshield2 "github.com/thachtb/solana-bridge/services-go/unshield"
	"strings"
)

const SHIELD = "Shield"
const UNSHIELD = "Unshield"
const INCOGNITO_PROXY = "5Tq3wvYAD6hRonCiUx62k37gELxxEABSYCkaqrSP3ztv"
const PROGRAM_ID = "BKGhwbiTHdUxcuWzZtDWyioRBieDEXTtgEk8u1zskZnk"
const SYNC_NATIVE_TAG = 0x11
const NEW_TOKEN_ACC = 0x1
const ACCCOUN_SIZE = 165

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

	// test shield tx
	recent, err := rpcClient.GetRecentBlockhash(context.Background(), rpc.CommitmentFinalized)
	if err != nil {
		panic(err)
	}

	fmt.Println("============ TEST SHIELD TOKEN ACCOUNT =============")

	shieldMakerTokenAccount := solana.MustPublicKeyFromBase58("5397KrEBCuEhdTjWF5B9xjVzGJR6MyxXLP3srbrWo2gD")
	vaultTokenAcc := solana.MustPublicKeyFromBase58("GKCSgiLWos1zEvNWKAEprngjdaazLUXMdM354Ag7CpX")

	incAddress := "12shR6fDe7ZcprYn6rjLwiLcL7oJRiek66ozzYu3B3rBxYXkqJeZYj6ZWeYy4qR4UHgaztdGYQ9TgHEueRXN7VExNRGB5t4auo3jTgXVBiLJmnTL5LzqmTXezhwmQvyrRjCbED5xW7yMMeeWarKa"
	shieldAmount := uint64(100000)
	shieldAccounts := []*solana.AccountMeta{
		solana.NewAccountMeta(shieldMakerTokenAccount, true, false),
		solana.NewAccountMeta(vaultTokenAcc, true, false),
		solana.NewAccountMeta(incognitoProxy, false, false),
		solana.NewAccountMeta(shieldMaker.PublicKey(), false, true),
		solana.NewAccountMeta(solana.TokenProgramID, false, false),
	}
	signers := []solana.PrivateKey{
		feePayer,
		shieldMaker,
	}

	shieldInstruction := shield.NewShield(
		incAddress,
		shieldAmount,
		program,
		shieldAccounts,
	)
	shieldInsGenesis := shieldInstruction.Build()
	if shieldInsGenesis == nil {
		panic("Build inst got error")
	}
	tx, err := solana.NewTransaction(
		[]solana.Instruction{
			shieldInsGenesis,
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(feePayer.PublicKey()),
	)
	if err != nil {
		panic(err)
	}
	sig, err := SignAndSendTx(tx, signers, rpcClient)
	if err != nil {
		panic(err)
	}
	spew.Dump(sig)

	fmt.Println("============ TEST CREATE NEW ASSOCIATE TOKEN ACCOUNT =============")
	mintPubkey := solana.MustPublicKeyFromBase58("EHheP6Wfyz65ve258TYQcfBHAAY4LsErnmXZozrgfvGr")
	newKey := solana.NewWallet()

	tx2, err := solana.NewTransaction(
		[]solana.Instruction{
			associatedtokenaccount.NewCreateInstruction(
				feePayer.PublicKey(),
				newKey.PublicKey(),
				mintPubkey,
			).Build(),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(feePayer.PublicKey()),
	)
	signers2 := []solana.PrivateKey{
		feePayer,
	}
	sig, err = SignAndSendTx(tx2, signers2, rpcClient)
	if err != nil {
		panic(err)
	}
	spew.Dump(sig)

	fmt.Println("============ TEST UNSHIELD TOKEN ACCOUNT =============")
	txBurn := "04932a9003db2990728ee2f8450463aa6c39ba4b2773573dc86938760eec7eba"
	vaultAcc := solana.MustPublicKeyFromBase58("FmARrhNZxzA6aPXGuxeM71DMTzwMUYxqvpC8kh1pLR8Y")
	signers3 := []solana.PrivateKey{
		feePayer,
	}
	vaultTokenAuthority, _, err := solana.FindProgramAddress(
		[][]byte{incognitoProxy.Bytes()},
		program,
	)
	if err != nil {
		panic(err)
	}
	unshieldAccounts := []*solana.AccountMeta{
		solana.NewAccountMeta(vaultTokenAcc, true, false),
		solana.NewAccountMeta(shieldMakerTokenAccount, true, false),
		solana.NewAccountMeta(vaultTokenAuthority, false, false),
		solana.NewAccountMeta(vaultAcc, false, false),
		solana.NewAccountMeta(incognitoProxy, false, false),
		solana.NewAccountMeta(solana.TokenProgramID, false, false),
	}

	unshield := unshield2.NewUnshield(
		txBurn,
		"getburnpbscprooffordeposittosc",
		"https://mainnet.incognito.org/fullnode",
		program,
		unshieldAccounts,
	)
	tx3, err := solana.NewTransaction(
		[]solana.Instruction{
			unshield.Build(),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(feePayer.PublicKey()),
	)
	sig, err = SignAndSendTx(tx3, signers3, rpcClient)
	if err != nil {
		//panic(err)
	}
	spew.Dump(sig)

	fmt.Println("==============================================================================")
	fmt.Println("==============================================================================")
	fmt.Println("============ TEST FULL FLOW SHIELD AND UNSHIELD FOR NATIVE TOKEN =============")
	fmt.Println("==============================================================================")
	solAmount := uint64(1e7)
	recent, err = rpcClient.GetRecentBlockhash(context.Background(), rpc.CommitmentFinalized)
	if err != nil {
		panic(err)
	}
	shieldNativeTokenAcc, _, err := solana.FindAssociatedTokenAddress(
		shieldMaker.PublicKey(),
		solana.SolMint,
	)
	if err != nil {
		panic(err)
	}

	fmt.Println(shieldNativeTokenAcc.String())

	// create new token account Token11..112 to shield
	shieldNativeTokenAccInst := associatedtokenaccount.NewCreateInstruction(
		feePayer.PublicKey(),
		shieldMaker.PublicKey(),
		solana.SolMint,
	).Build()

	vaultNativeTokenAcc, _, err := solana.FindAssociatedTokenAddress(
		vaultTokenAuthority,
		solana.SolMint,
	)

	if err != nil {
		panic(err)
	}
	fmt.Println(vaultNativeTokenAcc.String())

	vaultNativeTokenAccInst := associatedtokenaccount.NewCreateInstruction(
		feePayer.PublicKey(),
		vaultTokenAuthority,
		solana.SolMint,
	).Build()

	incAddress4 := "12shR6fDe7ZcprYn6rjLwiLcL7oJRiek66ozzYu3B3rBxYXkqJeZYj6ZWeYy4qR4UHgaztdGYQ9TgHEueRXN7VExNRGB5t4auo3jTgXVBiLJmnTL5LzqmTXezhwmQvyrRjCbED5xW7yMMeeWarKa"
	shieldAccounts4 := []*solana.AccountMeta{
		solana.NewAccountMeta(shieldNativeTokenAcc, true, false),
		solana.NewAccountMeta(vaultNativeTokenAcc, true, false),
		solana.NewAccountMeta(incognitoProxy, false, false),
		solana.NewAccountMeta(shieldMaker.PublicKey(), false, true),
		solana.NewAccountMeta(solana.TokenProgramID, false, false),
	}
	signers4 := []solana.PrivateKey{
		feePayer,
		shieldMaker,
	}

	shieldInstruction4 := shield.NewShield(
		incAddress4,
		solAmount,
		program,
		shieldAccounts4,
	)

	// build sync native token program
	syncNativeeInst := solana.NewInstruction(
		solana.TokenProgramID,
		[]*solana.AccountMeta{
			solana.NewAccountMeta(shieldNativeTokenAcc, true, false),
		},
		[]byte{SYNC_NATIVE_TAG},
	)

	tx4, err := solana.NewTransaction(
		[]solana.Instruction{
			shieldNativeTokenAccInst,
			vaultNativeTokenAccInst,
			system.NewTransferInstruction(
				solAmount,
				shieldMaker.PublicKey(),
				shieldNativeTokenAcc,
			).Build(),
			syncNativeeInst,
			shieldInstruction4.Build(),
		}[2:],
		recent.Value.Blockhash,
		solana.TransactionPayer(feePayer.PublicKey()),
	)
	if err != nil {
		panic(err)
	}
	sig4, err := SignAndSendTx(tx4, signers4, rpcClient)
	if err != nil {
		panic(err)
	}
	spew.Dump(sig4)

	// unshield sol
	nativeAccountToken, err := solana.WalletFromPrivateKeyBase58("YpRLgTL3DPc83MTjdsVE6ALv5RqUQt3jZ35aVQDxeAmJLqsDhXZFtPnFXganq6DfQ7Q91guGQjKc13YMVjyX8vP")
	if err != nil {
		panic(err)
	}

	unshieldAccounts5 := []*solana.AccountMeta{
		solana.NewAccountMeta(vaultNativeTokenAcc, true, false),
		solana.NewAccountMeta(nativeAccountToken.PublicKey(), true, false),
		solana.NewAccountMeta(vaultTokenAuthority, false, false),
		solana.NewAccountMeta(vaultAcc, false, false),
		solana.NewAccountMeta(incognitoProxy, false, false),
		solana.NewAccountMeta(solana.TokenProgramID, false, false),
		solana.NewAccountMeta(shieldMaker.PublicKey(), true, false),
	}

	signers5 := []solana.PrivateKey{
		feePayer,
		shieldMaker,
		nativeAccountToken.PrivateKey,
	}

	unshield5 := unshield2.NewUnshield(
		txBurn,
		"getburnpbscprooffordeposittosc",
		"https://mainnet.incognito.org/fullnode",
		program,
		unshieldAccounts5,
	)
	exemptLamport, err := rpcClient.GetMinimumBalanceForRentExemption(context.Background(), ACCCOUN_SIZE, rpc.CommitmentConfirmed)
	if err != nil {
		panic(err)
	}

	// build create new token acc
	newAccToken := solana.NewInstruction(
		solana.TokenProgramID,
		[]*solana.AccountMeta{
			solana.NewAccountMeta(nativeAccountToken.PublicKey(), true, false),
			solana.NewAccountMeta(solana.SolMint, false, false),
			solana.NewAccountMeta(vaultTokenAuthority, false, false),
			solana.NewAccountMeta(solana.SysVarRentPubkey, false, false),
		},
		[]byte{NEW_TOKEN_ACC},
	)

	tx5, err := solana.NewTransaction(
		[]solana.Instruction{
			system.NewCreateAccountInstruction(
				exemptLamport,
				ACCCOUN_SIZE,
				solana.TokenProgramID,
				feePayer.PublicKey(),
				nativeAccountToken.PublicKey(),
			).Build(),
			newAccToken,
			unshield5.Build(),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(feePayer.PublicKey()),
	)
	sig, err = SignAndSendTx(tx5, signers5, rpcClient)
	if err != nil {
		panic(err)
	}
	spew.Dump(sig)

	fmt.Println("============ TEST LISTEN TRANSFER TOKEN EVENT =============")
	//wsClient.ProgramSubscribeWithOpts(
	//	solana.SystemProgramID,
	//	rpc.CommitmentFinalized,
	//	solana.EncodingJSONParsed,
	//	[]rpc.RPCFilter{},
	//)

	fmt.Println("============ TEST LISTEN TRANSFER SOL EVENT =============")
	//wsClient.ProgramSubscribeWithOpts(
	//	solana.TokenProgramID,
	//	rpc.CommitmentFinalized,
	//	solana.EncodingJSONParsed,
	//	[]rpc.RPCFilter{},
	//)

	fmt.Println("============ TEST LISTEN SHIELD EVENT =============")
	// listen shield to vault logs
	{
		// Subscribe to log events that mention the provided pubkey:
		sub, err := wsClient.LogsSubscribeMentions(
			program,
			rpc.CommitmentFinalized,
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

func SignAndSendTx(tx *solana.Transaction, signers []solana.PrivateKey, rpcClient *rpc.Client) (solana.Signature, error) {
	_, err := tx.Sign(func(key solana.PublicKey) *solana.PrivateKey {
		for _, candidate := range signers {
			if candidate.PublicKey().Equals(key) {
				return &candidate
			}
		}
		return nil
	})
	if err != nil {
		fmt.Printf("unable to sign transaction: %v \n", err)
		return solana.Signature{}, err
	}
	// send tx
	signature, err := rpcClient.SendTransaction(context.Background(), tx)
	if err != nil {
		fmt.Printf("unable to send transaction: %v \n", err)
		return solana.Signature{}, err
	}
	return signature, nil
}
