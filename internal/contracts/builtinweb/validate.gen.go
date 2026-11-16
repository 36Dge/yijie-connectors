// Generated from builtin-web-retrieval/1. DO NOT EDIT.
package builtinweb

import (
	"bytes"
	"encoding/json"
	"errors"
	"github.com/santhosh-tekuri/jsonschema/v6"
	"regexp"
	"strings"
	"sync"
)

var once sync.Once
var compiled map[string]*jsonschema.Schema
var compileError error

func Validate(name string, raw []byte) error {
	once.Do(func() {
		var doc map[string]any
		if compileError = json.Unmarshal(Schema, &doc); compileError != nil {
			return
		}
		c := jsonschema.NewCompiler()
		c.AssertFormat()
		// RE2 lacks the source DateBound negative lookahead. This equivalent finite
		// year expression preserves 0001..9999 without adding another regex engine.
		c.UseRegexpEngine(func(pattern string) (jsonschema.Regexp, error) {
			pattern = strings.Replace(pattern, "^(?!0000)[0-9]{4}", "^([1-9][0-9]{3}|0[1-9][0-9]{2}|00[1-9][0-9]|000[1-9])", 1)
			return regexp.Compile(pattern)
		})
		const uri = "https://contracts.yijie.ai/local/builtin-web.json"
		if compileError = c.AddResource(uri, doc); compileError != nil {
			return
		}
		compiled = make(map[string]*jsonschema.Schema)
		for key := range doc["$defs"].(map[string]any) {
			compiled[key], compileError = c.Compile(uri + "#/$defs/" + key)
			if compileError != nil {
				return
			}
		}
	})
	if compileError != nil {
		return compileError
	}
	if compiled[name] == nil || !json.Valid(raw) {
		return errors.New("invalid web payload")
	}
	var value any
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	if decoder.Decode(&value) != nil || compiled[name].Validate(value) != nil {
		return errors.New("invalid web payload")
	}
	return nil
}
