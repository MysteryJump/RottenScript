sample:
	cd rotten-script-wasm/ && wasm-pack build --target=nodejs --dev && cd node-project && npm run start:sample

init:
	cd rotten-script-wasm/node-project && npm ci

clean:
	cargo clean && cd rotten-script-wasm && rm -r pkg && cd node-project/sample && rm -r dist
