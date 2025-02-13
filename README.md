# Token Microservice

## Getting started

### Installation

```sh
cargo install --git https://github.com/ferristhecrab/atom-token
```

### Running

#### Prerequisite
MongoDB running with [authentication set up](https://www.geeksforgeeks.org/how-to-enable-authentication-on-mongodb/);

```sh
CONFIG=/home/yourname/.config/atomics/token.json atom-token
```

Where `CONFIG` can be replaced with the location to the config file.

## API

Schema definition in [schema](./src/schema), exposed struct `Router` and `InternalRouter` in [router.rs](./src/router.rs) for squashed microservices.
