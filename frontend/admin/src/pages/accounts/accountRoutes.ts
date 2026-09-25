import type { AccountListQuery } from "../../types/account-query";
import { serializeAccountListQuery } from "../../types/account-query";

const ACCOUNT_LIST_PATH = "/admin/accounts";
const CREATE_ACCOUNT_PATH = "/admin/accounts/create";

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

export function buildEditAccountHref(
  accountId: string,
  returnTo: string,
): string {
  const params = new URLSearchParams({ return_to: returnTo });
  return `${ACCOUNT_LIST_PATH}/${encodeURIComponent(accountId)}/edit?${params.toString()}`;
}

export function resolveAccountReturnTo(search: string): string {
  const fallback = ACCOUNT_LIST_PATH;
  const requested = new URLSearchParams(search).get("return_to");

  if (requested === null) {
    return fallback;
  }

  try {
    const target = new URL(requested, window.location.origin);

    if (
      target.origin !== window.location.origin ||
      target.pathname !== ACCOUNT_LIST_PATH
    ) {
      return fallback;
    }

    return `${target.pathname}${target.search}`;
  } catch {
    return fallback;
  }
}
