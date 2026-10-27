# Primary Desktop worker output

FEAT-164 private build interface, additive output selection; model/tool wire, permissions and worker implementation are unchanged. The reviewed primary Desktop generation calls `scripts/build-market-worker.sh --output-root <generation>/worker` and explicitly sets `YIJIE_WORKER_TARGET_DIR=<generation>/worker-target`.

The optional output directory must already exist, be a real absolute owner-owned directory and not group/other writable. A generation manifest is exclusive: existing `current.json` rejects before compilation. Binary names remain SHA-256 content addressed, manifest schema2 and source checks remain. No argument preserves the previous `bin/market-worker` behavior. This is an output location option, not permission to overwrite an existing generation or substitute a binary. Existing default manifest/artifacts are retained by the Desktop migration procedure.
