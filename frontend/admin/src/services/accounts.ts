import { authService } from "./auth";
import {
  DEFAULT_ACCOUNT_LIST_QUERY,
  type AccountListQuery,
} from "../types/account-query";
import type {
  AccountListResponse,
  AccountStatus,
  AdministratorAccount,
} from "../types/account";

export interface ListAccountsOptions {
  signal?: AbortSignal;
}

export type AccountMutation = "deactivate" | "activate" | "restore";

type AccountsErrorCode =
  | "UNAUTHORIZED"
  | "FORBIDDEN"
  | "INVALID_RESPONSE"
  | "NETWORK"
  | "SERVER"
  | "UNSUPPORTED_MEDIA_TYPE"
  | "VALIDATION"
  | "NOT_FOUND"
  | "CONFLICT"
  | "UNKNOWN";

const apiOrigin = (import.meta.env.VITE_API_ORIGIN ?? "")
  .trim()
  .replace(/\/$/, "");

export class AccountsError extends Error {
  readonly code: AccountsErrorCode;
  readonly status: number;
  readonly problemCode: string | null;

  constructor(
    code: AccountsErrorCode,
    status: number,
    problemCode: string | null = null,
  ) {
    super(code);
    this.name = "AccountsError";
    this.code = code;
    this.status = status;
    this.problemCode = problemCode;
  }
}

function buildApiUrl(pathname: string): string {
  if (import.meta.env.PROD && window.location.protocol !== "https:") {
    throw new AccountsError("UNKNOWN", 0);
  }

  if (apiOrigin.length === 0) {
    return pathname;
  }

  let origin: URL;

  try {
    origin = new URL(apiOrigin);
  } catch {
    throw new AccountsError("UNKNOWN", 0);
  }

  if (import.meta.env.PROD && origin.protocol !== "https:") {
    throw new AccountsError("UNKNOWN", 0);
  }

  return new URL(
    pathname,
    `${origin.toString().replace(/\/$/, "")}/`,
  ).toString();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isString(value: unknown): value is string {
  return typeof value === "string";
}

function isAccountStatus(value: unknown): value is AccountStatus {
  return value === "active" || value === "inactive";
}

function isSafeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value);
}

async function readProblemCode(response: Response): Promise<string | null> {
  const contentType = response.headers.get("content-type") ?? "";

  if (!contentType.toLowerCase().includes("application/problem+json")) {
    return null;
  }

  try {
    const body: unknown = await response.json();

    if (isRecord(body) && isString(body.code)) {
      return body.code;
    }
  } catch {
    return null;
  }

  return null;
}

function mapResponseError(
  status: number,
  problemCode: string | null,
): AccountsError {
  switch (status) {
    case 400:
      return new AccountsError("VALIDATION", status, problemCode);
    case 401:
      return new AccountsError("UNAUTHORIZED", status, problemCode);
    case 403:
      return new AccountsError("FORBIDDEN", status, problemCode);
    case 404:
      return new AccountsError("NOT_FOUND", status, problemCode);
    case 409:
      return new AccountsError("CONFLICT", status, problemCode);
    case 415:
      return new AccountsError("UNSUPPORTED_MEDIA_TYPE", status, problemCode);
    case 422:
      return new AccountsError("VALIDATION", status, problemCode);
    default:
      if (status >= 500 && status <= 599) {
        return new AccountsError("SERVER", status, problemCode);
      }

      return new AccountsError("UNKNOWN", status, problemCode);
  }
}

function parseAccount(value: unknown): AdministratorAccount | null {
  if (!isRecord(value)) {
    return null;
  }

  if (
    !isString(value.id) ||
    !isString(value.email) ||
    !(value.display_name === null || isString(value.display_name)) ||
    !isAccountStatus(value.status) ||
    !isString(value.created_at) ||
    !isString(value.updated_at) ||
    !(value.deleted_at === null || isString(value.deleted_at))
  ) {
    return null;
  }

  return {
    id: value.id,
    email: value.email,
    displayName: value.display_name,
    status: value.status,
    createdAt: value.created_at,
    updatedAt: value.updated_at,
    deletedAt: value.deleted_at,
  };
}

function parseAccountListResponse(value: unknown): AccountListResponse {
  if (
    !isRecord(value) ||
    !Array.isArray(value.items) ||
    !isSafeInteger(value.page) ||
    value.page < 1 ||
    !isSafeInteger(value.page_size) ||
    value.page_size < 1 ||
    value.page_size > 100 ||
    !isSafeInteger(value.total) ||
    value.total < 0
  ) {
    throw new AccountsError("INVALID_RESPONSE", 200);
  }

  const items = value.items.map(parseAccount);

  if (items.some((item) => item === null)) {
    throw new AccountsError("INVALID_RESPONSE", 200);
  }

  return {
    items: items as AdministratorAccount[],
    page: value.page,
    pageSize: value.page_size,
    total: value.total,
  };
}

function buildListUrl(query: AccountListQuery): string {
  const params = new URLSearchParams();

  params.set("page", String(query.page));
  params.set("page_size", String(query.pageSize));

  if (query.status !== null) {
    params.set("status", query.status);
  }

  if (query.includeDeleted) {
    params.set("include_deleted", "true");
  }

  return `/admin/accounts?${params.toString()}`;
}

async function listAccounts(
  query: AccountListQuery,
  options: ListAccountsOptions = {},
): Promise<AccountListResponse> {
  const authorization = authService.getAuthorizationHeader();

  if (authorization === null) {
    throw new AccountsError("UNAUTHORIZED", 401, "UNAUTHORIZED");
  }

  let response: Response;

  try {
    response = await fetch(buildApiUrl(buildListUrl(query)), {
      method: "GET",
      headers: {
        Accept: "application/json",
        Authorization: authorization,
      },
      cache: "no-store",
      signal: options.signal,
    });
  } catch (error) {
    if (error instanceof DOMException && error.name === "AbortError") {
      throw error;
    }

    throw new AccountsError("NETWORK", 0);
  }

  if (response.status === 401) {
    const problemCode = await readProblemCode(response);
    authService.clearClientState();
    throw mapResponseError(401, problemCode);
  }

  if (!response.ok) {
    throw mapResponseError(response.status, await readProblemCode(response));
  }

  const contentType = response.headers.get("content-type")?.toLowerCase() ?? "";

  if (!contentType.includes("application/json")) {
    throw new AccountsError("INVALID_RESPONSE", response.status);
  }

  let body: unknown;

  try {
    body = await response.json();
  } catch {
    throw new AccountsError("INVALID_RESPONSE", response.status);
  }

  return parseAccountListResponse(body);
}

async function mutateAccount(
  action: AccountMutation,
  accountId: string,
): Promise<void> {
  const authorization = authService.getAuthorizationHeader();

  if (authorization === null) {
    throw new AccountsError("UNAUTHORIZED", 401, "UNAUTHORIZED");
  }

  let response: Response;

  try {
    response = await fetch(
      buildApiUrl(`/admin/accounts/${encodeURIComponent(accountId)}/${action}`),
      {
        method: "POST",
        headers: {
          Accept: "application/json",
          Authorization: authorization,
        },
        cache: "no-store",
      },
    );
  } catch {
    throw new AccountsError("NETWORK", 0);
  }

  if (response.status === 401) {
    const problemCode = await readProblemCode(response);
    authService.clearClientState();
    throw mapResponseError(401, problemCode);
  }

  if (!response.ok) {
    throw mapResponseError(response.status, await readProblemCode(response));
  }
}

export const accountsService = {
  async list(
    query: AccountListQuery = DEFAULT_ACCOUNT_LIST_QUERY,
    options: ListAccountsOptions = {},
  ): Promise<AccountListResponse> {
    return listAccounts(query, options);
  },

  async mutate(action: AccountMutation, accountId: string): Promise<void> {
    return mutateAccount(action, accountId);
  },
};
