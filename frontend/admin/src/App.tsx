import { useEffect, useSyncExternalStore } from "react";
import { AdminShell } from "./layouts/AdminShell";
import { AccountDetailPage } from "./pages/accounts/AccountDetailPage";
import { AccountEditPage } from "./pages/accounts/AccountEditPage";
import { AccountRoutePlaceholderPage } from "./pages/accounts/AccountRoutePlaceholderPage";
import { AccountsPage } from "./pages/accounts/AccountsPage";
import { LoginPage } from "./pages/login/LoginPage";
import { authService } from "./services/auth";
import { navigate, usePathname, type AppRoute } from "./services/navigation";

interface RouteTransitionProps {
  message: string;
}

function RouteTransition({ message }: RouteTransitionProps) {
  return (
    <main className="route-transition" aria-live="polite">
      <p role="status" aria-live="polite">
        {message}
      </p>
    </main>
  );
}

function isAccountDetailRoute(pathname: string): boolean {
  return /^\/admin\/accounts\/[^/]+$/.test(pathname);
}

function isAccountEditRoute(pathname: string): boolean {
  return /^\/admin\/accounts\/[^/]+\/edit$/.test(pathname);
}

function isAdminRoute(pathname: string): boolean {
  return (
    pathname === "/admin" ||
    pathname === "/admin/accounts" ||
    pathname === "/admin/accounts/create" ||
    isAccountDetailRoute(pathname) ||
    isAccountEditRoute(pathname)
  );
}

export default function App() {
  const pathname = usePathname();

  const authSnapshot = useSyncExternalStore(
    authService.subscribe,
    authService.getSnapshot,
  );

  const { authStatus } = authSnapshot;

  useEffect(() => {
    void authService.bootstrap();
  }, []);

  let redirectTarget: AppRoute | null = null;

  if (authStatus === "authenticated") {
    if (pathname === "/login") {
      redirectTarget = "/admin";
    } else if (!isAdminRoute(pathname)) {
      redirectTarget = "/admin";
    }
  } else if (authStatus === "unauthenticated") {
    if (pathname !== "/login") {
      redirectTarget = "/login";
    }
  } else if (authStatus === "authentication-error" && pathname !== "/login") {
    redirectTarget = "/login";
  }

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

    if (isAccountDetailRoute(pathname)) {
      document.title = "Account | Admin Application";
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

  if (redirectTarget !== null) {
    return <RouteTransition message="Redirecting…" />;
  }

  if (authStatus === "unknown") {
    return <RouteTransition message="Checking authentication…" />;
  }

  if (pathname === "/login") {
    return (
      <LoginPage
        onLogin={authService.login}
        bootstrapError={authStatus === "authentication-error"}
        onRetryBootstrap={authService.retryBootstrap}
      />
    );
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
        <AccountRoutePlaceholderPage />
      </AdminShell>
    );
  }

  if (isAccountDetailRoute(pathname)) {
    return (
      <AdminShell onLogout={authService.logout}>
        <AccountDetailPage />
      </AdminShell>
    );
  }

  if (isAccountEditRoute(pathname)) {
    return (
      <AdminShell onLogout={authService.logout}>
        <AccountEditPage />
      </AdminShell>
    );
  }

  if (pathname === "/admin") {
    return <AdminShell onLogout={authService.logout} />;
  }

  return <RouteTransition message="Redirecting…" />;
}
