import { useEffect, useSyncExternalStore } from "react";
import { AdminShell } from "./layouts/AdminShell";
import { AccountRoutePlaceholderPage } from "./pages/accounts/AccountRoutePlaceholderPage";
import { AccountsPage } from "./pages/accounts/AccountsPage";
import { LoginPage } from "./pages/login/LoginPage";
import { authService } from "./services/auth";
import { navigate, usePathname } from "./services/navigation";

function RouteTransition() {
  return (
    <main className="route-transition" aria-live="polite">
      Redirecting…
    </main>
  );
}

function AuthenticationPending() {
  return (
    <main className="route-transition" role="status" aria-live="polite">
      Checking authentication…
    </main>
  );
}

function AuthenticationErrorView() {
  return (
    <main className="route-transition" aria-live="polite">
      <section aria-labelledby="authentication-error-title">
        <h1 id="authentication-error-title">Authentication unavailable</h1>
        <p>
          We could not verify your existing authentication session. Please try
          again.
        </p>
        <button
          className="primary-button"
          type="button"
          onClick={() => void authService.retryBootstrap()}
        >
          Retry authentication
        </button>
      </section>
    </main>
  );
}

function isAccountEditRoute(pathname: string): boolean {
  return /^\/admin\/accounts\/[^/]+\/edit$/.test(pathname);
}

function isAdminRoute(pathname: string): boolean {
  return (
    pathname === "/admin" ||
    pathname === "/admin/accounts" ||
    pathname === "/admin/accounts/create" ||
    isAccountEditRoute(pathname)
  );
}

export default function App() {
  const pathname = usePathname();
  const authenticationSnapshot = useSyncExternalStore(
    authService.subscribe,
    authService.getSnapshot,
  );

  useEffect(() => {
    void authService.bootstrap();
  }, []);

  const authStatus = authenticationSnapshot.authStatus;

  const redirectTarget =
    authStatus === "unknown" || authStatus === "authentication-error"
      ? null
      : pathname === "/login" && authStatus === "authenticated"
        ? "/admin"
        : isAdminRoute(pathname) && authStatus === "unauthenticated"
          ? "/login"
          : pathname !== "/login" && !isAdminRoute(pathname)
            ? authStatus === "authenticated"
              ? "/admin"
              : "/login"
            : null;

  useEffect(() => {
    if (redirectTarget !== null) {
      navigate(redirectTarget, { replace: true });
    }
  }, [redirectTarget]);

  useEffect(() => {
    if (pathname === "/admin/accounts") {
      document.title = "Accounts | Admin Application";
      return;
    }

    if (pathname === "/admin/accounts/create") {
      document.title = "Create Account | Admin Application";
      return;
    }

    if (isAccountEditRoute(pathname)) {
      document.title = "Edit Account | Admin Application";
      return;
    }

    if (pathname === "/admin") {
      document.title = "Admin Application";
      return;
    }

    document.title = "Sign in | Admin Application";
  }, [pathname]);

  if (authStatus === "unknown") {
    return <AuthenticationPending />;
  }

  if (authStatus === "authentication-error") {
    if (pathname === "/login") {
      return (
        <LoginPage
          onLogin={authService.login}
          bootstrapError
          onRetryBootstrap={authService.retryBootstrap}
        />
      );
    }

    return <AuthenticationErrorView />;
  }

  if (redirectTarget !== null) {
    return <RouteTransition />;
  }

  if (pathname === "/login") {
    return <LoginPage onLogin={authService.login} />;
  }

  if (pathname === "/admin/accounts") {
    return (
      <AdminShell onLogout={authService.logout}>
        <AccountsPage />
      </AdminShell>
    );
  }

  if (pathname === "/admin/accounts/create") {
    return (
      <AdminShell onLogout={authService.logout}>
        <AccountRoutePlaceholderPage mode="create" />
      </AdminShell>
    );
  }

  if (isAccountEditRoute(pathname)) {
    return (
      <AdminShell onLogout={authService.logout}>
        <AccountRoutePlaceholderPage mode="edit" />
      </AdminShell>
    );
  }

  if (pathname === "/admin") {
    return <AdminShell onLogout={authService.logout} />;
  }

  return <RouteTransition />;
}
