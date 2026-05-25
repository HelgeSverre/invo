---
id: decision-3
title: Support provider base_url for all providers
date: '2026-05-25 15:31'
status: Accepted
---
## Context

The config model includes `llm.providers[].base_url` on provider entries. The field should not be aspirational or provider-specific unless the config model says so explicitly.

Current implementation status before TASK-26:

- Ollama uses `base_url`, falling back to `http://localhost:11434` when omitted.
- OpenAI provider construction exists, but `base_url` handling needs to be verified and wired through.
- Anthropic and Mistral are accepted provider names but not implemented yet.

The earlier open question was whether to support OpenAI-compatible base URLs or document `base_url` as Ollama-only. That ambiguity is now resolved: `base_url` is intended to be an active provider setting for every provider exposed by `invo`.

This matters because users may need custom endpoints for:

- self-hosted Ollama instances,
- OpenAI-compatible gateways or proxies,
- enterprise/private API gateways,
- local model routers,
- provider-compatible deployments with non-default hosts.

For an agent-friendly CLI, accepting a config field but silently ignoring it is worse than rejecting it. If `base_url` is accepted on provider entries, it should either be honored or produce a clear error when the selected provider/client cannot support it.

Related backlog items:

- TASK-26 — Support provider base_url for all providers
- TASK-30 — Implement Anthropic and Mistral providers

## Decision

`llm.providers[].base_url` is an active provider setting for all providers that `invo` exposes.

Expected behavior:

1. If `base_url` is configured on the selected provider entry, provider client construction uses that URL.
2. If `base_url` is omitted, the provider uses its normal default endpoint.
3. Provider override behavior does not change: `--provider <name>` selects a configured provider entry by name, and that selected entry's `base_url` is honored.
4. Documentation, config schema descriptions, `config doctor`, and `agent-context` should treat `base_url` as active for all providers, not as parsed-only or Ollama-only.
5. Tests should verify `base_url` is passed into each provider path without requiring live network calls.

Provider-specific notes:

| Provider | `base_url` policy |
| -------- | ----------------- |
| `ollama` | Optional. Defaults to `http://localhost:11434`. Custom values point at another Ollama-compatible endpoint. |
| `openai` | Optional. Defaults to the provider/client default. Custom values point at OpenAI-compatible API endpoints when supported by Rig. |
| `anthropic` | Optional once implemented. Defaults to the provider/client default. Custom values point at Anthropic-compatible endpoints when supported by Rig. |
| `mistral` | Optional once implemented. Defaults to the provider/client default. Custom values point at Mistral-compatible endpoints when supported by Rig. |

If Rig lacks a clean custom-base-url API for a provider, implementation should prefer a clear runtime/config error for that provider over silently ignoring the field. However, the product intent remains that `base_url` is supported for all providers.

## Consequences

Positive consequences:

- Provider configuration is consistent and easier to explain.
- Users can route traffic through local gateways, proxies, or enterprise endpoints.
- `agent-context` and config schema can present `base_url` as a real active setting instead of a caveat.
- Future provider work has a clear requirement: no provider should accept and ignore `base_url`.

Tradeoffs and risks:

- Some provider-compatible endpoints may differ in structured output support, auth, model naming, or schema behavior. Supporting `base_url` does not guarantee every compatible-looking endpoint behaves identically.
- Tests may need seams around provider client construction to verify URLs without live API calls.
- Provider-specific Rig APIs may differ, so implementation may not be identical across providers.
- TASK-30 should inherit this decision for Anthropic and Mistral implementation.

Follow-up actions:

1. Implement TASK-26 with all-provider `base_url` semantics.
2. Inspect Rig provider APIs for custom base URL support across Ollama, OpenAI, Anthropic, and Mistral.
3. Add tests that prove configured `base_url` values are used by provider construction or produce clear unsupported-client errors rather than being ignored.
4. Update README, config schema descriptions, `config doctor`, and `agent-context` to match the accepted all-provider policy.
5. Ensure TASK-30 includes `base_url` behavior when adding Anthropic and Mistral runtime support.
