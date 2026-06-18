PORT = 2000
CARGO_EXEC = cargo run
MAP_PATH = test.yaml

all:
	cd backend && cargo run $(PORT) $(MAP_PATH) & cd frontend && cargo run $(PORT) & wait

serveur:
	cd backend && $(CARGO_EXEC) $(PORT) $(MAP_PATH)

client-gui:
	cd frontend && $(CARGO_EXEC) $(PORT)

client-cli:
	nc localhost $(PORT)

fclean:
	rm -rf backend/target frontend/target || true

re: fclean all

.PHONY: all install serveur client-gui fclean re