FROM node:22.23.2-bookworm-slim AS web-build
WORKDIR /source
COPY package.json package-lock.json ./
RUN npm ci
COPY apps/web ./apps/web
COPY packages/design-system ./packages/design-system
COPY tsconfig.json vitest.config.ts playwright.config.ts ./
RUN npm run build:web

FROM rust:1.98.0-bookworm AS api-build
ARG BUILD_SHA=dev
ENV BUILD_SHA=${BUILD_SHA}
WORKDIR /source
COPY Cargo.toml Cargo.lock ./
COPY services/api ./services/api
RUN cargo build --release --manifest-path services/api/Cargo.toml

FROM debian:bookworm-slim AS runtime
RUN groupadd --system reminderproof && useradd --system --gid reminderproof --create-home reminderproof
WORKDIR /app
COPY --from=api-build /source/target/release/reminder-proof-api /usr/local/bin/reminder-proof-api
COPY --from=web-build /source/dist ./dist
USER reminderproof
ENV PORT=8080
EXPOSE 8080
CMD ["reminder-proof-api"]
