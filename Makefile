CARGO_INSTALL = cargo build
CARGO_EXEC = cargo run
MAP_PATH = test.yaml
PORT = 2000

all:
	@trap 'kill 0' EXIT INT TERM; \
	cd backend && cargo build --quiet; cd ..; \
	cd frontend && cargo build --quiet; cd ..; \
	(cd backend && cargo run --quiet $(PORT) $(MAP_PATH)) & \
	sleep 1; \
	(cd frontend && cargo run --quiet $(PORT)) & \
	wait

install:
	cd backend && $(CARGO_INSTALL) & \
	cd frontend && $(CARGO_INSTALL) & \
	cd client_cli && $(CARGO_INSTALL)

run-server:
	cd backend && $(CARGO_EXEC) $(PORT) $(MAP_PATH)

run-client-gui:
	cd frontend && $(CARGO_EXEC) $(PORT)

run-client:
	cd client_cli && $(CARGO_EXEC) $(PORT)

clean:
	rm -rf backend/target || true \
	frontend/target || true \
	client_cli/target || true

lint:
	cd backend && cargo clippy
	cd frontend && cargo clippy
	cd client_cli && cargo clippy

re: clean all

.PHONY: all install run-server run-client-gui run-client clean re
