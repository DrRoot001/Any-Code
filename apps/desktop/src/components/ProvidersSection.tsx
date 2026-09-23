import { useCallback, useEffect, useState } from "react";
import { providerCommands, type ProviderStatus } from "../lib/tauri";

/**
 * What a provider's row says. "Connected" would be a claim about reachability that this
 * screen can't make — a local server may not be running — so it reports configuration.
 */
function statusLabel(p: ProviderStatus): string {
  if (p.needsEndpoint) return p.endpoint ? `Configured · ${p.endpoint}` : "Not configured";
  if (!p.requiresKey) return "Local";
  return p.hasKey ? "Key saved" : "No key";
}

export default function ProvidersSection() {
  const [providers, setProviders] = useState<ProviderStatus[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [endpointDrafts, setEndpointDrafts] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState<string | null>(null);

  const refresh = useCallback(() => {
    providerCommands
      .listProviders()
      .then((list) => {
        setProviders(list);
        setError(null);
      })
      .catch((reason) => setError(String(reason)));
  }, []);

  useEffect(refresh, [refresh]);

  const save = async (id: string) => {
    const key = drafts[id]?.trim();
    if (!key) return;
    setBusy(id);
    try {
      await providerCommands.setProviderKey(id, key);
      setDrafts((d) => ({ ...d, [id]: "" }));
      refresh();
    } catch (reason) {
      setError(`Could not save the key for ${id}: ${String(reason)}`);
    } finally {
      setBusy(null);
    }
  };

  const saveEndpoint = async (id: string, baseUrl: string | null) => {
    setBusy(id);
    try {
      await providerCommands.setProviderEndpoint(id, baseUrl);
      setEndpointDrafts((d) => ({ ...d, [id]: "" }));
      refresh();
    } catch (reason) {
      // The runtime rejects plain http to another machine; say why rather than failing mutely.
      setError(`Could not save the endpoint: ${String(reason)}`);
    } finally {
      setBusy(null);
    }
  };

  const remove = async (id: string) => {
    setBusy(id);
    try {
      await providerCommands.removeProviderKey(id);
      refresh();
    } catch (reason) {
      setError(`Could not remove the key for ${id}: ${String(reason)}`);
    } finally {
      setBusy(null);
    }
  };

  if (error) {
    return (
      <p className="danger" role="alert">
        {error}
      </p>
    );
  }
  if (!providers) {
    return <p className="muted">Loading providers…</p>;
  }

  return (
    <div className="provider-list">
      {providers.map((p) => (
        <div key={p.id} className="provider-row">
          <div className="provider-row-heading">
            <span>{p.name}</span>
            <span className={p.ready ? "provider-status provider-status--ok" : "muted"}>
              {statusLabel(p)}
            </span>
          </div>
          {p.needsEndpoint && (
            <div className="provider-row-controls">
              <input
                type="url"
                placeholder={p.endpoint ?? "https://…/v1, or http://localhost:1234/v1"}
                value={endpointDrafts[p.id] ?? ""}
                onChange={(e) => setEndpointDrafts((d) => ({ ...d, [p.id]: e.target.value }))}
                aria-label={`${p.name} base URL`}
              />
              <button
                className="button"
                onClick={() => saveEndpoint(p.id, endpointDrafts[p.id]?.trim() ?? "")}
                disabled={busy === p.id || !endpointDrafts[p.id]?.trim()}
              >
                Save
              </button>
              {p.endpoint && (
                <button
                  className="button"
                  onClick={() => saveEndpoint(p.id, null)}
                  disabled={busy === p.id}
                >
                  Remove
                </button>
              )}
            </div>
          )}
          {(p.requiresKey || p.needsEndpoint) && (
            <div className="provider-row-controls">
              <input
                type="password"
                placeholder={p.needsEndpoint ? "API key (optional)" : "API key"}
                value={drafts[p.id] ?? ""}
                onChange={(e) => setDrafts((d) => ({ ...d, [p.id]: e.target.value }))}
                aria-label={`${p.name} API key`}
              />
              <button
                className="button"
                onClick={() => save(p.id)}
                disabled={busy === p.id || !drafts[p.id]?.trim()}
              >
                Save
              </button>
              {p.hasKey && (
                <button className="button" onClick={() => remove(p.id)} disabled={busy === p.id}>
                  Remove
                </button>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
