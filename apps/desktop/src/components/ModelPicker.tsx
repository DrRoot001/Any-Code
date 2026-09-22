import type { ModelDefinition, ProviderStatus } from "../lib/tauri";

/** The provider/model dropdown pair shared by the Chat and Agent surfaces. */
export default function ModelPicker({
  connected,
  provider,
  onProviderChange,
  models,
  modelsError,
  model,
  onModelChange,
  disabled,
}: {
  connected: ProviderStatus[];
  provider: string | null;
  onProviderChange: (id: string) => void;
  models: ModelDefinition[] | null;
  modelsError: string | null;
  model: string | null;
  onModelChange: (id: string) => void;
  disabled?: boolean;
}) {
  return (
    <div className="panel-subheader chat-toolbar">
      <label className="chat-select">
        Provider
        <select
          value={provider ?? ""}
          onChange={(e) => onProviderChange(e.target.value)}
          disabled={disabled}
        >
          {connected.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </label>
      <label className="chat-select">
        Model
        {modelsError ? (
          <span className="danger">{modelsError}</span>
        ) : (
          <select
            value={model ?? ""}
            onChange={(e) => onModelChange(e.target.value)}
            disabled={disabled || !models}
          >
            {models?.map((m) => (
              <option key={m.id} value={m.id}>
                {m.displayName}
              </option>
            ))}
          </select>
        )}
      </label>
    </div>
  );
}
