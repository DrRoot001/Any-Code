import { applyTheme, type ThemeName } from "@anycode/design-tokens";
import { commands } from "../lib/tauri";
import { useWorkbenchStore } from "../state/workbenchStore";
import { useState } from "react";
import { Icon } from "./Icons";
import MemorySection from "./MemorySection";
import ProvidersSection from "./ProvidersSection";
import { useDialogFocus } from "../hooks/useDialogFocus";

const THEMES: ThemeName[] = ["system", "light", "dark", "high-contrast"];

export default function SettingsPanel({
  theme,
  onThemeChange,
  onClose,
}: {
  theme: ThemeName;
  onThemeChange: (theme: ThemeName) => void;
  onClose: () => void;
}) {
  const workspace = useWorkbenchStore((s) => s.workspace);
  const [error, setError] = useState<string | null>(null);
  const dialogRef = useDialogFocus<HTMLDivElement>(true, onClose);

  const changeTheme = (next: ThemeName) => {
    applyTheme(next);
    onThemeChange(next);
    commands
      .setTheme(next)
      .then(() => setError(null))
      .catch((reason) =>
        setError(`Theme changed for this session but could not be saved: ${String(reason)}`),
      );
  };

  return (
    <div className="overlay settings-overlay" onClick={onClose}>
      <div
        className="settings"
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="settings-heading">
          <h2 id="settings-title">Settings</h2>
          <button className="icon-button" onClick={onClose} aria-label="Close settings">
            <Icon name="close" />
          </button>
        </div>

        {error && (
          <p className="danger" role="alert">
            {error}
          </p>
        )}
        <section>
          <h3>Appearance</h3>
          {THEMES.map((name) => (
            <label key={name} className="setting-option">
              <input
                type="radio"
                name="theme"
                checked={theme === name}
                onChange={() => changeTheme(name)}
              />{" "}
              {name.replace("high-contrast", "High contrast").replace(/^./, (c) => c.toUpperCase())}
            </label>
          ))}
        </section>

        <section>
          <h3>Providers</h3>
          <ProvidersSection />
        </section>

        <section>
          <h3>Memory</h3>
          <MemorySection />
        </section>

        <section>
          <h3>Workspace</h3>
          <p style={{ fontSize: "0.8rem", color: "var(--ac-text-muted)", wordBreak: "break-all" }}>
            {workspace?.path ?? "No workspace open"}
          </p>
        </section>
      </div>
    </div>
  );
}
