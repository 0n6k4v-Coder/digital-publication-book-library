import { useEffect, useRef, useState } from "react";
import type { MouseEvent, ReactNode } from "react";
import { navigate, usePathname } from "../services/navigation";

interface AdminShellProps {
  onLogout: () => Promise<void>;
  children?: ReactNode;
}

export function AdminShell({ onLogout, children }: AdminShellProps) {
  const pathname = usePathname();

  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [logoutError, setLogoutError] = useState<string | null>(null);
  const [isMobileNavigationOpen, setIsMobileNavigationOpen] = useState(false);

  const mainRef = useRef<HTMLElement>(null);
  const mobileMenuButtonRef = useRef<HTMLButtonElement>(null);
  const mobileCloseButtonRef = useRef<HTMLButtonElement>(null);
  const hasMountedRef = useRef(false);

  const isAccountsActive = pathname === "/admin/accounts";
  const isAdminHomeActive = pathname === "/admin";

  useEffect(() => {
    if (!hasMountedRef.current) {
      hasMountedRef.current = true;
      return;
    }

    mainRef.current?.focus();
  }, [pathname]);

  useEffect(() => {
    if (!isMobileNavigationOpen) {
      return;
    }

    mobileCloseButtonRef.current?.focus();

    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        event.preventDefault();
        setIsMobileNavigationOpen(false);
        mobileMenuButtonRef.current?.focus();
      }
    }

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isMobileNavigationOpen]);

  useEffect(() => {
    if (!isMobileNavigationOpen) {
      return;
    }

    setIsMobileNavigationOpen(false);
    mobileMenuButtonRef.current?.focus();
  }, [pathname]);

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

  function closeMobileNavigation(): void {
    setIsMobileNavigationOpen(false);
    mobileMenuButtonRef.current?.focus();
  }

  function handleNavigation(
    event: MouseEvent<HTMLAnchorElement>,
    target: "/admin" | "/admin/accounts",
  ): void {
    if (
      event.defaultPrevented ||
      event.button !== 0 ||
      event.metaKey ||
      event.ctrlKey ||
      event.shiftKey ||
      event.altKey
    ) {
      return;
    }

    event.preventDefault();

    setIsMobileNavigationOpen(false);
    navigate(target);
  }

  return (
    <div className="admin-shell">
      <a className="admin-skip-link" href="#admin-main-content">
        Skip to main content
      </a>

      <header className="admin-mobile-header">
        <p className="admin-mobile-header__title">Admin Application</p>

        <button
          ref={mobileMenuButtonRef}
          className="admin-mobile-header__menu"
          type="button"
          aria-controls="admin-sidebar"
          aria-expanded={isMobileNavigationOpen}
          aria-label={
            isMobileNavigationOpen
              ? "Close admin navigation"
              : "Open admin navigation"
          }
          onClick={() => setIsMobileNavigationOpen((current) => !current)}
        >
          <span aria-hidden="true">
            {isMobileNavigationOpen ? "Close" : "Menu"}
          </span>
        </button>
      </header>

      <aside
        id="admin-sidebar"
        className={`admin-sidebar${
          isMobileNavigationOpen ? " admin-sidebar--open" : ""
        }`}
        aria-label="Admin application"
        aria-busy={isLoggingOut}
      >
        <div className="admin-sidebar__top">
          <div className="admin-sidebar__brand">
            <p className="eyebrow">Admin Application</p>
            <h1 className="admin-sidebar__title">
              Digital Publication &amp; Book Library
            </h1>
          </div>

          <button
            ref={mobileCloseButtonRef}
            className="admin-sidebar__close"
            type="button"
            aria-label="Close admin navigation"
            onClick={closeMobileNavigation}
          >
            Close
          </button>
        </div>

        <nav
          className="admin-sidebar__nav"
          aria-label="Admin feature navigation"
        >
          <ul className="admin-sidebar__nav-list">
            <li>
              <a
                className={`admin-nav-link${
                  isAdminHomeActive ? " admin-nav-link--active" : ""
                }`}
                href="/admin"
                aria-current={isAdminHomeActive ? "page" : undefined}
                onClick={(event) => handleNavigation(event, "/admin")}
              >
                Dashboard
              </a>
            </li>

            <li>
              <a
                className={`admin-nav-link${
                  isAccountsActive ? " admin-nav-link--active" : ""
                }`}
                href="/admin/accounts"
                aria-current={isAccountsActive ? "page" : undefined}
                onClick={(event) => handleNavigation(event, "/admin/accounts")}
              >
                Accounts
              </a>
            </li>
          </ul>
        </nav>

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

      {isMobileNavigationOpen ? (
        <button
          className="admin-sidebar-backdrop"
          type="button"
          aria-label="Dismiss admin navigation"
          onClick={closeMobileNavigation}
        />
      ) : null}

      <main
        ref={mainRef}
        id="admin-main-content"
        className="admin-main"
        aria-label="Main content"
        tabIndex={-1}
      >
        {children}
      </main>
    </div>
  );
}
