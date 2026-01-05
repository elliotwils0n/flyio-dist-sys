test-echo:
	cargo build
	~/Downloads/maelstrom/maelstrom test \
		-w echo \
		--bin ./target/debug/echo \
		--node-count 1 \
		--time-limit 10

test-unique-id-generation:
	cargo build
	~/Downloads/maelstrom/maelstrom test \
		-w unique-ids \
		--bin ./target/debug/unique_ids \
		--time-limit 30 \
		--rate 1000 \
		--node-count 3 \
		--availability total \
		--nemesis partition

test-broadcast:
	cargo build
	~/Downloads/maelstrom/maelstrom test \
		-w broadcast \
		--bin ./target/debug/broadcast \
		--node-count 5 \
		--time-limit 20 \
		--rate 10 \
		--nemesis partition

debug:
	~/Downloads/maelstrom/maelstrom serve
