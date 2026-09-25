import { useSyncExternalStore } from "react";

export type AppRoute = "/login" | "/admin";

function normalizePathname(pathname: string): string {
  if (pathname.length > 1 && pathname.endsWith("/")) {
    return pathname.slice(0, -1);
  }

  return pathname;
}

export function getPathname(): string {
  return normalizePathname(window.location.pathname);
}

export function navigate(pathname: AppRoute): void {
  if (getPathname() === pathname) {
    return;
  }

  window.history.replaceState({}, "", pathname);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

export function usePathname(): string {
  return useSyncExternalStore(
    (listener) => {
      window.addEventListener("popstate", listener);
      return () => window.removeEventListener("popstate", listener);
    },
    getPathname,
    () => "/",
  );
}