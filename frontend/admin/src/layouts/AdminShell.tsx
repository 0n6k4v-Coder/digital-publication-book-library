import { useState } from "react";

interface AdminShellProps {
  onLogout: () => Promise<void>;
}

export function AdminShell({ onLogout }: AdminShellProps) {
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [logoutError, setLogoutError] = useState<string | null>(null);

  async function handleLogout(): Promise<void> {
    if (isLoggingOut) {
      return;
    }

    setLogoutError(null);
    setIsLoggingOut(true);

    try {
      await onLogout();
    } catch {
      setLogoutError("We could not sign you out. Please try again.");
    } finally {
      setIsLoggingOut(false);
    }
  }

  return (
    <div className="admin-shell">
      <aside
        className="admin-sidebar"
        aria-label="Admin application"
        aria-busy={isLoggingOut}
      >
        <div>
          <p className="eyebrow">Admin Application</p>
          <h1 className="admin-sidebar__title">
            Digital Publication &amp; Book Library
          </h1>
        </div>

        <div className="admin-sidebar__footer">
          {logoutError !== null ? (
            <div className="form-alert form-alert--compact" role="alert">
              {logoutError}
            </div>
          ) : null}

          <button
            className="secondary-button"
            type="button"
            disabled={isLoggingOut}
            onClick={handleLogout}
          >
            {isLoggingOut ? "Signing out…" : "Logout"}
          </button>

          {isLoggingOut ? (
            <p
              className="admin-sidebar__status"
              role="status"
              aria-live="polite"
            >
              Signing out…
            </p>
          ) : null}
        </div>
      </aside>

      <main className="admin-main" aria-label="Main content" />
    </div>
  );
}
