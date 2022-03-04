package unshield

import (
	"errors"
	"github.com/ethereum/go-ethereum/crypto"

	"encoding/hex"
)

const CBridgeSigSz = 65

func decodeECDSASig(sig []byte) (
	v byte,
	r []byte,
	s []byte,
	err error,
) {
	if len(sig) != CBridgeSigSz {
		err = errors.New("wrong input")
		return
	}
	v = byte(sig[64])
	r = sig[:32]
	s = sig[32:64]
	return
}

func toByte32(s []byte) [32]byte {
	a := [32]byte{}
	copy(a[:], s)
	return a
}

func decode(s string) []byte {
	d, _ := hex.DecodeString(s)
	return d
}

func decode32(s string) [32]byte {
	return toByte32(decode(s))
}

func keccak256(b ...[]byte) [32]byte {
	h := crypto.Keccak256(b...)
	r := [32]byte{}
	copy(r[:], h)
	return r
}