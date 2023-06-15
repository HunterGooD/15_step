
run:
	# cargo dylint bevy_lint
	cargo run --features bevy/dynamic_linking # bevy 10.0
	# cargo run --features bevy/dynamic # bevy 9.0

build_win:
	cargo build --target=x86_64-pc-windows-gnu --release

build:
	cargo build --release #build_all: build_win #	build_gnu

