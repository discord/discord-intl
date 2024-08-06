# Dockerfiles for building in CI

These dockerfiles help speed up CI by pre-installing all of the necessary build environment tools, including:

- node
- pnpm
- rust
- zig
- cargo-zigbuild
- cargo-xwin

It also includes a pre-downloaded cache of most dependencies (cargo and npm) to speed up installation times.
