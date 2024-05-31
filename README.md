# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:
```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

---

[//]: # "The following is the Frontend Template's README.md"

# Soroban Frontend in Astro

A Frontend Template suitable for use with `soroban contract init --frontend-template`, powered by [Astro](https://astro.build/).

# Getting Started

- `cp .env.example .env`
- `npm install`
- `npm run dev`

# How it works

If you look in [package.json](./package.json), you'll see that the `start` & `dev` scripts first run the [`initialize.js`](./initialize.js) script. This script loops over all contracts in `contracts/*` and, for each:

1. Deploys to a local network (_needs to be running with `docker run` or `soroban network start`_)
2. Saves contract IDs to `.soroban/contract-ids`
3. Generates TS bindings for each into the `packages` folder, which is set up as an [npm workspace](https://docs.npmjs.com/cli/v10/configuring-npm/package-json#workspaces)
4. Create a file in `src/contracts` that imports the contract client and initializes it for the `standalone` network.

You're now ready to import these initialized contract clients in your [Astro templates](https://docs.astro.build/en/core-concepts/astro-syntax/) or your [React, Svelte, Vue, Alpine, Lit, and whatever else JS files](https://docs.astro.build/en/core-concepts/framework-components/#official-ui-framework-integrations). You can see an example of this in [index.astro](./src/pages/index.astro).

# Deploying to a Soroban Network

Contract
```
soroban contract install \
--network testnet \
--source alice \
--wasm target/wasm32-unknown-unknown/release/lumen_finance_contract.wasm
118b27fcc06f76417943dff46b5b498f8d1280f9fcd1ec3e89984cbe9ab4169f
```
```
lumenfinance git:(main) ✗ soroban contract deploy \
--wasm-hash 118b27fcc06f76417943dff46b5b498f8d1280f9fcd1ec3e89984cbe9ab4169f \
--source alice \
--network testnet
CDUS3S5OX4EKRXMSXBV5JALOYHRW2B356JVXSIOYQPXLAP6LNYVZ7WLJ
```
```
lumenfinance git:(main) ✗ soroban contract invoke \
--id CDUS3S5OX4EKRXMSXBV5JALOYHRW2B356JVXSIOYQPXLAP6LNYVZ7WLJ \
--source alice \
--network testnet \
-- \
initialize \
--token_wasm_hash c6fe61fb6c64cbe3e23cb52b059d43545875cf1bc2d396c6d901cfa51a712033 \
--usdc CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA \
--admin GDTWEIIS33YYOP366W3FAJYZAAZ4INTXNRN7YUYNNNAIFNR4XBH3CL4B \
--insurance GDG4ZVJUYP2ZOMHAD72TG3FMVFA4M3A23GDK5WR2AXARVEUSJBOEXGL2 
```
Token (mock USDC)
```
soroban contract deploy \                                 
  --wasm-hash c6fe61fb6c64cbe3e23cb52b059d43545875cf1bc2d396c6d901cfa51a712033 \
  --source alice \
  --network testnet
CDR44SASFO5WOJAFAEXO4PAH2TIQCX2ZF3NTWTZP3MJM6QIDGMRDI4P5
```
