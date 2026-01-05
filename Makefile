test-echo:
	cargo build --release
	~/Downloads/maelstrom/maelstrom test \
		-w echo \
		--bin ./target/release/echo \
		--node-count 1 \
		--time-limit 10

test-unique-id-generation:
	cargo build --release
	~/Downloads/maelstrom/maelstrom test \
		-w unique-ids \
		--bin ./target/release/unique_ids \
		--time-limit 30 \
		--rate 1000 \
		--node-count 3 \
		--availability total \
		--nemesis partition

test-broadcast:
	cargo build --release
	~/Downloads/maelstrom/maelstrom test \
		-w broadcast \
		--bin ./target/release/broadcast \
		--node-count 5 \
		--time-limit 20 \
		--rate 10 \
		--nemesis partition

test-broadcast-perf:
	cargo build --release
	~/Downloads/maelstrom/maelstrom test \
		-w broadcast \
		--bin ./target/release/broadcast \
		--node-count 25 \
		--time-limit 20 \
		--rate 100 \
		--latency 100

debug:
	~/Downloads/maelstrom/maelstrom serve
