import { useEffect, useSyncExternalStore } from "react";
import { AdminShell } from "./layouts/AdminShell";
import { AccountDetailPage } from "./pages/accounts/AccountDetailPage";
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

  const isAuthenticated = useSyncExternalStore(
    authService.subscribe,
    authService.getSnapshot,
  );

  const redirectTarget =
    pathname === "/login" && isAuthenticated
      ? "/admin"
      : isAdminRoute(pathname) && !isAuthenticated
        ? "/login"
        : pathname !== "/login" && !isAdminRoute(pathname)
          ? isAuthenticated
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
        <AccountDetailPage />
      </AdminShell>
    );
  }

  if (pathname === "/admin") {
    return <AdminShell onLogout={authService.logout} />;
  }

  return <RouteTransition />;
}
