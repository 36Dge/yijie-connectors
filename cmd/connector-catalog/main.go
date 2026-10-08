// connector-catalog validates and emits the safe product directory for packaging.
package main

import (
	"os"

	"github.com/36Dge/yijie-connectors/catalog"
)

func main() {
	content, err := catalog.PublicJSON()
	if err != nil {
		_, _ = os.Stderr.WriteString("Managed connector catalog validation failed.\n")
		os.Exit(1)
	}
	_, _ = os.Stdout.Write(content)
}
