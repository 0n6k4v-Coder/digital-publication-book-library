import { useEffect, useSyncExternalStore } from "react";
import { AdminShell } from "./layouts/AdminShell";
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

export default function App() {
  const pathname = usePathname();
  const isAuthenticated = useSyncExternalStore(
    authService.subscribe,
    authService.getSnapshot,
  );

  const redirectTarget =
    pathname === "/login" && isAuthenticated
      ? "/admin"
      : pathname === "/admin" && !isAuthenticated
        ? "/login"
        : pathname !== "/login" && pathname !== "/admin"
          ? isAuthenticated
            ? "/admin"
            : "/login"
          : null;

  useEffect(() => {
    if (redirectTarget !== null) {
      navigate(redirectTarget);
    }
  }, [redirectTarget]);

  if (redirectTarget !== null) {
    return <RouteTransition />;
  }

  if (pathname === "/login") {
    return <LoginPage onLogin={authService.login} />;
  }

  if (pathname === "/admin") {
    return <AdminShell onLogout={authService.logout} />;
  }

  return <RouteTransition />;
}
