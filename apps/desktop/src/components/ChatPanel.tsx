import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useProviderModel } from "../hooks/useProviderModel";
import { providerCommands, type ChatMessage } from "../lib/tauri";
import { Icon } from "./Icons";
import ModelPicker from "./ModelPicker";

/**
 * Proves Phase 2's exit condition end to end: pick a provider, pick a model, send a
 * message, watch it stream back — and switching providers changes nothing here except
 * which two dropdown values are selected. All provider-specific behavior stays inside
 * the Rust adapters; this component only ever sees the normalized chat/message shape.
 */
export default function ChatPanel() {
  const picker = useProviderModel();
  const { provider, model } = picker;
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [pending, setPending] = useState<string | null>(null);
  const [sendError, setSendError] = useState<string | null>(null);
  const sessionId = useMemo(() => crypto.randomUUID(), []);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [messages]);

  const send = useCallback(async () => {
    if (!provider || !model || !draft.trim() || pending) return;
    const next = [...messages, { role: "user" as const, content: draft.trim() }];
    setMessages(next);
    setDraft("");
    setSendError(null);

    const requestId = crypto.randomUUID();
    setPending(requestId);
    setMessages([...next, { role: "assistant", content: "" }]);

    // Subscribe before sending: a request that fails at once (a local server that isn't
    // running) must not report its error into a channel nobody is listening on yet.
    const offs: UnlistenFn[] = [];
    const detach = () => offs.splice(0).forEach((off) => off());
    offs.push(
      await listen<{ text: string }>(`chat:delta:${requestId}`, (event) => {
        setMessages((current) => {
          const updated = [...current];
          const last = updated[updated.length - 1];
          if (last?.role === "assistant") {
            updated[updated.length - 1] = { ...last, content: last.content + event.payload.text };
          }
          return updated;
        });
      }),
      await listen(`chat:done:${requestId}`, () => {
        setPending(null);
        detach();
      }),
      await listen<{ message: string }>(`chat:error:${requestId}`, (event) => {
        setSendError(event.payload.message);
        setPending(null);
        detach();
      }),
    );

    try {
      await providerCommands.sendChat(requestId, provider, model, sessionId, next);
    } catch (error) {
      // Never started, so no terminal event is coming.
      setSendError(String(error));
      setPending(null);
      setMessages(next);
      detach();
    }
  }, [provider, model, draft, pending, messages, sessionId]);

  if (picker.providersError) {
    return (
      <div className="empty-state">
        <div>
          <strong>Could not load providers</strong>
          {picker.providersError}
        </div>
      </div>
    );
  }
  if (!picker.providers) {
    return <div className="empty-state muted">Loading providers…</div>;
  }

  if (picker.connected.length === 0) {
    return (
      <div className="empty-state">
        <div>
          <strong>No provider connected</strong>
          Add an API key or an endpoint in Settings → Providers, or install Ollama for a local model.
        </div>
      </div>
    );
  }

  return (
    <div className="panel">
      <ModelPicker
        connected={picker.connected}
        provider={provider}
        onProviderChange={picker.setProvider}
        models={picker.models}
        modelsError={picker.modelsError}
        model={model}
        onModelChange={picker.setModel}
      />

      <div className="panel-scroll chat-messages" ref={scrollRef}>
        {messages.length === 0 && <p className="muted">Ask something to try {provider}.</p>}
        {messages.map((m, i) => (
          <div key={i} className={`chat-message chat-message--${m.role}`}>
            <span className="chat-role">{m.role}</span>
            <p>{m.content || (pending && i === messages.length - 1 ? "…" : "")}</p>
          </div>
        ))}
      </div>

      {sendError && (
        <p className="danger chat-error" role="alert">
          {sendError}
        </p>
      )}

      <form
        className="chat-input"
        onSubmit={(e) => {
          e.preventDefault();
          send();
        }}
      >
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder={model ? `Message ${model}…` : "Select a model first"}
          disabled={!model || !!pending}
          aria-label="Chat message"
        />
        <button
          type="submit"
          className="icon-button"
          disabled={!model || !draft.trim() || !!pending}
          aria-label="Send message"
        >
          <Icon name="send" />
        </button>
      </form>
    </div>
  );
}
