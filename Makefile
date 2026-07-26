CARGO_INSTALL = cargo build
CARGO_EXEC = cargo run
MAP_PATH = test.yaml
PORT = 2000

all:
	@trap 'kill 0' EXIT INT TERM; \
	cd backend && cargo run $(PORT) $(MAP_PATH) & \
	cd frontend && cargo run $(PORT) \
	& wait

install:
	cd backend && $(CARGO_INSTALL) & \
	cd frontend && $(CARGO_INSTALL)

run-server:
	cd backend && $(CARGO_EXEC) $(PORT) $(MAP_PATH)

run-client-gui:
	cd frontend && $(CARGO_EXEC) $(PORT)

run-client:
	nc localhost $(PORT)

clean:
	rm -rf backend/target || true \
	frontend/target || true

lint:
	cd backend && cargo clippy
	cd frontend && cargo clippy

re: clean all

.PHONY: all install run-server run-client-gui run-client clean re
