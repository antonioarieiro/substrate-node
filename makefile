alice:

	./target/release/parachain-template-node \
	--alice \
	--collator \
	--force-authoring \
	--chain rococo-local-parachain-2000-raw.json \
	--base-path /tmp/parachain/alice \
	--port 30337 \
	--ws-port 9947 \
	--rpc-cors=all \
	--rpc-methods=Unsafe \
	-- \
	--execution wasm \
	--chain rococo_local.json \

wasm:
	./target/release/parachain-template-node export-genesis-wasm --chain rococo-local-parachain-2000-raw.json > validation_files/para-2000-wasm

genesis:
	./target/release/parachain-template-node export-genesis-state --chain rococo-local-parachain-2000-raw.json > validation_files/para-2000-genesis

purge:
	cargo run --release -- purge-chain

chain-spec-plain:
	./target/release/parachain-template-node build-spec --disable-default-bootnode > rococo-local-parachain-plain.json

chain-spec-raw:
	./target/release/parachain-template-node build-spec --chain rococo-local-parachain-plain.json --raw --disable-default-bootnode > rococo-local-parachain-2000-raw.json


