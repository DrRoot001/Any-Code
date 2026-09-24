type IconName =
  | "files"
  | "branch"
  | "terminal"
  | "settings"
  | "command"
  | "folder"
  | "close"
  | "chat"
  | "plug"
  | "send"
  | "history";

export function Icon({ name, size = 16 }: { name: IconName; size?: number }) {
  const paths: Record<IconName, React.ReactNode> = {
    files: (
      <>
        <path d="M5 3h9l4 4v14H5z" />
        <path d="M14 3v5h5M2 7v14h11" />
      </>
    ),
    branch: (
      <>
        <circle cx="6" cy="5" r="2" />
        <circle cx="6" cy="19" r="2" />
        <circle cx="18" cy="7" r="2" />
        <path d="M6 7v10M8 7h4a6 6 0 0 1 6 6v-4" />
      </>
    ),
    terminal: (
      <>
        <path d="m4 7 4 4-4 4M11 16h7" />
      </>
    ),
    settings: (
      <>
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.6v-.2h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z" />
      </>
    ),
    command: (
      <>
        <path d="M8 6h8M8 12h8M8 18h5" />
        <circle cx="4" cy="6" r="1" />
        <circle cx="4" cy="12" r="1" />
        <circle cx="4" cy="18" r="1" />
      </>
    ),
    folder: <path d="M3 6h7l2 2h9v11H3z" />,
    close: <path d="m7 7 10 10M17 7 7 17" />,
    chat: (
      <>
        <path d="M4 5h16v11H9l-5 4z" />
      </>
    ),
    plug: (
      <>
        <path d="M9 3v6M15 3v6M6 9h12l-1 4a5 5 0 0 1-5 4 5 5 0 0 1-5-4z M12 17v4" />
      </>
    ),
    send: <path d="M3 11 21 3l-6 18-4-8-8-2z" />,
    history: (
      <>
        <path d="M3 12a9 9 0 1 0 3-6.7" />
        <path d="M3 4v4h4M12 8v5l3 2" />
      </>
    ),
  };
  return (
    <svg
      aria-hidden="true"
      className="icon"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {paths[name]}
    </svg>
  );
}
