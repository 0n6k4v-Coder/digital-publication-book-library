import type { AccountListQuery } from "../../types/account-query";
import { serializeAccountListQuery } from "../../types/account-query";

const ACCOUNT_LIST_PATH = "/admin/accounts";
const CREATE_ACCOUNT_PATH = "/admin/accounts/create";
const ACCOUNT_LIST_QUERY_KEYS = new Set([
  "page",
  "page_size",
  "status",
  "include_deleted",
]);

function isPositiveInteger(value: string): boolean {
  if (!/^\d+$/.test(value)) {
    return false;
  }

  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed >= 1;
}

function isSupportedAccountListSearch(params: URLSearchParams): boolean {
  const seen = new Set<string>();

  for (const [key, value] of params.entries()) {
    if (!ACCOUNT_LIST_QUERY_KEYS.has(key) || seen.has(key)) {
      return false;
    }

    seen.add(key);

    switch (key) {
      case "page":
        if (!isPositiveInteger(value)) {
          return false;
        }
        break;
      case "page_size": {
        if (!isPositiveInteger(value) || Number(value) > 100) {
          return false;
        }
        break;
      }
      case "status":
        if (value !== "active" && value !== "inactive") {
          return false;
        }
        break;
      case "include_deleted":
        if (value !== "true") {
          return false;
        }
        break;
      default:
        return false;
    }
  }

  return true;
}

export function buildAccountListHref(query: AccountListQuery): string {
  return `${ACCOUNT_LIST_PATH}${serializeAccountListQuery(query)}`;
}

export function buildCreateAccountHref(returnTo: string): string {
  if (returnTo === ACCOUNT_LIST_PATH) {
    return CREATE_ACCOUNT_PATH;
  }

  const params = new URLSearchParams({ return_to: returnTo });
  return `${CREATE_ACCOUNT_PATH}?${params.toString()}`;
}

export function buildAccountDetailHref(
  accountId: string,
  returnTo: string = ACCOUNT_LIST_PATH,
): string {
  const path = `${ACCOUNT_LIST_PATH}/${encodeURIComponent(accountId)}`;

  if (returnTo === ACCOUNT_LIST_PATH) {
    return path;
  }

  const params = new URLSearchParams({ return_to: returnTo });
  return `${path}?${params.toString()}`;
}

export function buildEditAccountHref(
  accountId: string,
  returnTo: string,
): string {
  const params = new URLSearchParams({ return_to: returnTo });
  return `${ACCOUNT_LIST_PATH}/${encodeURIComponent(accountId)}/edit?${params.toString()}`;
}

export function buildDeleteAccountHref(
  accountId: string,
  returnTo: string,
): string {
  const params = new URLSearchParams({ return_to: returnTo });
  return `${ACCOUNT_LIST_PATH}/${encodeURIComponent(accountId)}/delete?${params.toString()}`;
}

export function resolveAccountReturnTo(search: string): string {
  const fallback = ACCOUNT_LIST_PATH;
  const params = new URLSearchParams(search);
  const requestedValues = params.getAll("return_to");

  if (requestedValues.length === 0) {
    return fallback;
  }

  if (requestedValues.length !== 1) {
    return fallback;
  }

  const requested = requestedValues[0];

  try {
    const target = new URL(requested, window.location.origin);

    if (
      target.origin !== window.location.origin ||
      target.pathname !== ACCOUNT_LIST_PATH ||
      target.username.length > 0 ||
      target.password.length > 0 ||
      target.hash.length > 0 ||
      !isSupportedAccountListSearch(target.searchParams)
    ) {
      return fallback;
    }

    return `${target.pathname}${target.search}`;
  } catch {
    return fallback;
  }
}
