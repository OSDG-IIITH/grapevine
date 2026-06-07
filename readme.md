## grapevine

grapevine is a course and advisor review platform for IIIT Hyderabad.

### Setup

If you use **Nix**, you can use flake.nix for a development shell with required dependencies.

Otherwise,you will need:
- [Rust](https://www.rust-lang.org/)
- [Bun](https://bun.sh/)
- [Docker](https://www.docker.com/)
- [sqlx-cli](https://github.com/launchbadge/sqlx) (`cargo install sqlx-cli --no-default-features --features postgres`)

#### Environment Variables

Copy `.env.example` over to `.env`. Make changes if needed.
```
cp .env.example .env
```

#### Database

Start the postgres container:
```
docker compose -f docker-compose.dev.yml up -d
```

Run the migrations:
```
sqlx migrate run
```

Seed the database:
```
cd server
cargo run --bin seed
cd ..
```

#### Run

To run the backend:
```
cd server
cargo run
```
Runs ar `http://localhost:3000`

To run the frontend:
```
cd web
bun install
bun run dev
```
Runs at `http://localhost:5173`
