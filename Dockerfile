FROM node:22.23.2-bookworm-slim AS web-build
ARG BUILD_SHA=dev
ENV BUILD_SHA=${BUILD_SHA}
WORKDIR /source
COPY package.json package-lock.json ./
RUN npm ci
COPY apps/web ./apps/web
COPY packages/design-system ./packages/design-system
COPY tsconfig.json vitest.config.ts playwright.config.ts ./
RUN npm run build:web

FROM rust:1-slim AS api-build
ARG BUILD_SHA=dev
ARG GIT_SHA=dev
ARG SOURCE_COMMIT=dev
ENV BUILD_SHA=${BUILD_SHA}
WORKDIR /source
COPY Cargo.toml Cargo.lock ./
COPY services/api ./services/api
RUN cargo build --release --manifest-path services/api/Cargo.toml

FROM debian:bookworm-slim AS runtime
ARG BUILD_SHA=dev
ARG GIT_SHA=dev
ARG SOURCE_COMMIT=dev
RUN groupadd --system reminderproof \
  && useradd --system --gid reminderproof --create-home reminderproof \
  && mkdir /data /durable /backups \
  && chown reminderproof:reminderproof /data /durable /backups
WORKDIR /app
COPY --from=api-build /source/target/release/reminder-proof-api /usr/local/bin/reminder-proof-api
COPY --from=web-build /source/dist ./dist
USER reminderproof
ENV PORT=8080
ENV DATA_DIR=/data
ENV DURABLE_DIR=/durable
ENV BACKUP_DIR=/backups
EXPOSE 8080
CMD ["reminder-proof-api"]
