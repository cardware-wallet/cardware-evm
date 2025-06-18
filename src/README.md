# cardware-evm
EDGE CASE ERROR:

If you see an error that looks like this:

  Internal error occurred: Command "/opt/homebrew/Cellar/llvm/17.0.2/bin/clang-17" "-O3" "-ffunction-sections" "-fdata-sections" "-fPIC" "--target=wasm32-unknown-unknown" "-Wall" "-Wextra" "-o" "/Users/dom/Documents/Rust/scl_wallet/target/wasm32-unknown-unknown/release/build/rust-crypto-wasm-71b19d49c9aa1fe2/out/src/util_helpers.o" "-c" "src/util_helpers.c" with args "clang-17" did not execute successfully (status code exit status: 1).

This is caused by a rust compilation order of operations problem

The solution is to:

	1) Exit your terminal window and quit terminal

	2) Navigate to your working directory and run "wasm-pack build" this will fail with a C_Linker error

	3) Export your C_Linker using: "export TARGET_CC=/opt/homebrew/Cellar/llvm/20.1.5/bin/clang-20"

	4) Re-run "wasm-pack build" this time it will work (black magic!)
