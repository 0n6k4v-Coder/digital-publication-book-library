import { useEffect, useSyncExternalStore } from "react";
import { AdminShell } from "./layouts/AdminShell";
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

function isAdminRoute(pathname: string): boolean {
  return pathname === "/admin" || pathname === "/admin/accounts";
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

  if (pathname === "/admin") {
    return <AdminShell onLogout={authService.logout} />;
  }

  return <RouteTransition />;
}
