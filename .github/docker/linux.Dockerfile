# Start from the cargo-zigbuild base image to automatically get Rust and Zig
FROM messense/cargo-zigbuild
# Add Node for package development.
FROM node:20.6.1-bookworm-slim

# Make the working directory the repo root to reference pnpm-lock.yaml and
# Cargo.lock as expected.
WORKDIR ../../

# Install pnpm. Keep this version in sync with the monorepo
RUN corepack prepare pnpm@9.0.6 --activate
RUN corepack enable

# Do a pre-installation to get most dependencies cached in a local store that \
# gets copied into the contain
RUN pnpm config set store-dir ./.cache/pnpm-store
RUN pnpm install

# Do the same for Cargo dependencies
ENV CARGO_HOME = "./.cache/cargo-home"
RUN cargo fetch

COPY ./.cache ./.cache