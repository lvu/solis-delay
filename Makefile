test:
	env RUST_BACKTRACE=0 cargo test

image:
	docker build -t solis-delay .