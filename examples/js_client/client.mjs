import { rpc } from "@stellar/stellar-sdk";

const url = process.env.SOROBAN_RPC_URL ?? "http://127.0.0.1:8000/soroban-rpc";
const server = new rpc.Server(url, { allowHttp: true });

const health = await server.getHealth();
if (health.status !== "healthy" || health.latestLedger !== 1000) {
  throw new Error(`unexpected health response: ${JSON.stringify(health)}`);
}

const network = await server.getNetwork();
if (network.passphrase !== "Test SDF Network ; September 2015") {
  throw new Error(`unexpected network response: ${JSON.stringify(network)}`);
}

console.log(JSON.stringify({ health, network }));
