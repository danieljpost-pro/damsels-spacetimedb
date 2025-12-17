# Damsels - SpacetimeDB Game Backend

A session-based, real-time multiplayer game built with Rust and SpacetimeDB.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [SpacetimeDB CLI](https://spacetimedb.com/install)

## Quick Start

### Install SpacetimeDB CLI

```bash
curl -sSf https://install.spacetimedb.com | sh
```

### Build the Module

```bash
spacetime build
```

### Start Local SpacetimeDB Server

```bash
spacetime start
```

### Publish the Module

```bash
spacetime publish --server local damsels
```

### Test the Hello World Reducer

```bash
# Call the say_hello reducer
spacetime call --server local damsels say_hello

# Check the logs
spacetime logs --server local damsels
```

## Project Structure

```
damsels-spacetimedb/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── lib.rs          # SpacetimeDB module (tables & reducers)
└── README.md
```

## Related Repositories

- **damsels-pingora** - Infrastructure (Helm charts, K8s manifests)

