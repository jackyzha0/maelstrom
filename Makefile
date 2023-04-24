results:
	./maelstrom-binary/maelstrom serve

echo:
	cargo build --bin echo
	./maelstrom-binary/maelstrom test -w echo --bin ./target/debug/echo --log-stderr --time-limit 10

id:
	cargo build --bin id
	./maelstrom-binary/maelstrom test -w unique-ids --bin ./target/debug/id --time-limit 30 --log-stderr --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast:
	cargo build --bin broadcast
	./maelstrom-binary/maelstrom test -w broadcast --bin ./target/debug/broadcast --log-stderr --node-count 5 --time-limit 20 --rate 10 --nemesis partition

broadcast-efficient:
	cargo build --bin broadcast
	./maelstrom-binary/maelstrom test -w broadcast --bin ./target/debug/broadcast --log-stderr --node-count 25 --time-limit 20 --rate 100 --latency 100

g-counter:
	cargo build --bin g-counter
	./maelstrom-binary/maelstrom test -w g-counter --bin ./target/debug/g-counter --log-stderr --node-count 3 --rate 100 --time-limit 20 --nemesis partition
