package main

import (
	"encoding/json"
	"fmt"
	"os"

	pbbstream "github.com/dfuse-io/pbgo/dfuse/bstream/v1"
	"github.com/golang/protobuf/proto"
	"github.com/streamingfast/dbin"
	pbeth "github.com/streamingfast/firehose-ethereum/types/pb/sf/ethereum/type/v2"
)

func check(e error) {
    if e != nil {
        panic(e)
    }
}
func main() {
	reader, err := dbin.NewFileReader("./example0017686312.dbin")
	check(err)
	_, header, err := reader.ReadHeader()
	check(err)
	fmt.Println(header)
	message, err := reader.ReadMessage()
	check(err)

	pbBlock := new(pbbstream.Block)
	proto.Unmarshal(message, pbBlock)

	fmt.Println(pbBlock.Timestamp.AsTime())

	block := new(pbeth.Block)
	err = proto.Unmarshal(pbBlock.PayloadBuffer, block)
	check(err)

	// fmt.Println(block.TransactionTraces)

	outFile, err := os.Create("./out.json")
	check(err)
	jsonData, err := json.Marshal(block)
	check(err)

	outFile.Write(jsonData)
}
