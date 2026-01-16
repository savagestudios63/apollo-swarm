import type { TaskContext, TaskHandler } from "./orchestrator";

/**
 * Example echo handler — simply returns the payload URI so downstream
 * consumers can observe the task lifecycle end-to-end. Replace with an
 * LLM call (Anthropic, OpenAI, etc.) or external tool invocation.
 */
export const echoHandler: TaskHandler = async (ctx: TaskContext) => {
  return `echo:${ctx.payloadUri}`;
};

/**
 * Factory for an LLM-backed handler. The `invoke` callback receives the
 * payload URI (typically pointing at IPFS / Arweave JSON) and returns a
 * result URI pointing at the artefact the agent produced.
 */
export function llmHandler(
  invoke: (payloadUri: string) => Promise<string>,
): TaskHandler {
  return async (ctx) => invoke(ctx.payloadUri);
}
