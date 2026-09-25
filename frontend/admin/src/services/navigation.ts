import { useSyncExternalStore } from "react";

export type AppRoute = "/login" | "/admin" | "/admin/accounts";

function normalizePathname(pathname: string): string {
  if (pathname.length > 1 && pathname.endsWith("/")) {
    return pathname.slice(0, -1);
  }

  return pathname;
}

export function getPathname(): string {
  return normalizePathname(window.location.pathname);
}

export function navigateTo(
  destination: string,
  options: { replace?: boolean } = {},
): void {
  if (!destination.startsWith("/")) {
    throw new Error(
      "Client navigation destinations must be same-origin paths.",
    );
  }

  const current = `${window.location.pathname}${window.location.search}${window.location.hash}`;

  if (current === destination) {
    return;
  }

  if (options.replace === true) {
    window.history.replaceState({}, "", destination);
  } else {
    window.history.pushState({}, "", destination);
  }

  window.dispatchEvent(new PopStateEvent("popstate"));
}

export function navigate(
  pathname: AppRoute,
  options: { replace?: boolean } = {},
): void {
  navigateTo(pathname, options);
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
