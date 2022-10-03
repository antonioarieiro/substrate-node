alice:
	  ./target/release/parachain-template-node \
		--alice \
		--collator \
		--force-authoring \
		--chain rococo-local-parachain-2000-raw.json \
		--base-path /tmp/parachain/alice \
		--port 30340  \
		--ws-port 9950 \
		--rpc-cors=all \
		--rpc-methods=Unsafe \
		-- \
		--execution wasm \
		--chain rococo_local.json \
		--telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
		--validator \
		--rpc-methods Unsafe \
		--name kinergy \


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


