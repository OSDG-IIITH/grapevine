## grapevine

grapevine is a course and advisor review platform for IIIT Hyderabad.

### Setup

If you use **Nix**, you can use `flake.nix` for a development shell with required dependencies.

Otherwise,you will need:
- [Rust](https://www.rust-lang.org/)
- [Bun](https://bun.sh/)
- [Docker](https://www.docker.com/)
- [sqlx-cli](https://github.com/launchbadge/sqlx) (`cargo install sqlx-cli --no-default-features --features postgres`)

#### Environment Variables

Copy `.env.example` over to `.env`.
```
cp .env.example .env
```
Make changes if needed (add your email to `MODERATOR_EMAILS` to test moderator capabilities, or modify `ALLOWED_EMAIL_DOMAINS` to add custom domains for registration. Defaults to `students.iiit.ac.in,research.iiit.ac.in`).

#### Database

Start the postgres container:
```
docker compose -f docker-compose.dev.yml up -d
```

Run the migrations:
```
cd server
sqlx migrate run
cd ..
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
Runs at `http://localhost:3000`

To run the frontend:
```
cd web
bun install
bun run dev
```
Runs at `http://localhost:5173`
