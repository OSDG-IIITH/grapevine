FROM oven/bun:slim AS builder
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile
COPY . .
ARG BASE_PATH=""
ENV BASE_PATH=$BASE_PATH
ARG PUBLIC_API_URL="/api"
ENV PUBLIC_API_URL=$PUBLIC_API_URL
RUN bun run build

FROM caddy:2-alpine
COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/build /srv
