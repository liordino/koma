// Transport-contract smoke: the TS client compiles against the shared proto and can build the
// unary gRPC-Web transport the server serves (koma.v1.Koma/Ping). Real UI arrives in M1.
import { createClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import { KomaService } from "./gen/koma/v1/koma_pb";

const transport = createConnectTransport({
  baseUrl: "http://127.0.0.1:8080",
  fetch: (input, init) =>
    fetch(input, init === undefined ? undefined : { ...init }),
});

const client = createClient(KomaService, transport);
export async function ping(): Promise<string> {
  return (await client.ping({})).message;
}
