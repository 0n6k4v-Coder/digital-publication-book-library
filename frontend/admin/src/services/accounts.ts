import { authService } from "./auth";
import type {
  AccountListQuery,
  AccountStatusFilter,
} from "../types/account-query";
import { DEFAULT_ACCOUNT_LIST_QUERY } from "../types/account-query";
import type {
  AccountListResponse,
  AccountStatus,
  AdministratorAccount,
} from "../types/account";

interface ListAccountsOptions {
  signal?: AbortSignal;
}

export type AccountMutation = "deactivate" | "activate" | "restore";

type AccountsErrorCode =
  | "UNAUTHORIZED"
  | "FORBIDDEN"
  | "NOT_FOUND"
  | "CONFLICT"
  | "VALIDATION"
  | "UNSUPPORTED_MEDIA_TYPE"
  | "SERVER"
  | "NETWORK"
  | "INVALID_RESPONSE"
  | "UNKNOWN";

interface ProblemDetails {
  code: string | null;
}

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

function isAccountStatusFilter(value: unknown): value is AccountStatusFilter {
  return value === "active" || value === "inactive";
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
    typeof value.page !== "number" ||
    !Number.isInteger(value.page) ||
    typeof value.page_size !== "number" ||
    !Number.isInteger(value.page_size) ||
    typeof value.total !== "number" ||
    !Number.isInteger(value.total) ||
    value.page < 1 ||
    value.page_size < 1 ||
    value.page_size > 100 ||
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

function buildListSearch(query: AccountListQuery): string {
  const params = new URLSearchParams();
  params.set("page", String(Math.max(1, query.page)));
  params.set("page_size", String(Math.min(100, Math.max(1, query.pageSize))));

  if (query.status !== null && isAccountStatusFilter(query.status)) {
    params.set("status", query.status);
  }

  if (query.includeDeleted) {
    params.set("include_deleted", "true");
  }

  return params.toString();
}

async function readProblemDetails(response: Response): Promise<ProblemDetails> {
  const contentType = response.headers.get("content-type") ?? "";

  if (!contentType.toLowerCase().includes("application/problem+json")) {
    return { code: null };
  }

  try {
    const body: unknown = await response.json();

    if (
      isRecord(body) &&
      typeof body.code === "string" &&
      body.code.length > 0
    ) {
      return { code: body.code };
    }
  } catch {
    return { code: null };
  }

  return { code: null };
}

async function toAccountsError(response: Response): Promise<AccountsError> {
  const { code: problemCode } = await readProblemDetails(response);

  if (response.status === 401) {
    authService.clearClientState();
    return new AccountsError("UNAUTHORIZED", 401, problemCode);
  }

  if (response.status === 403) {
    return new AccountsError("FORBIDDEN", 403, problemCode);
  }

  if (response.status === 404) {
    return new AccountsError("NOT_FOUND", 404, problemCode);
  }

  if (response.status === 409) {
    return new AccountsError("CONFLICT", 409, problemCode);
  }

  if (response.status === 415) {
    return new AccountsError("UNSUPPORTED_MEDIA_TYPE", 415, problemCode);
  }

  if (response.status === 400 || response.status === 422) {
    return new AccountsError("VALIDATION", response.status, problemCode);
  }

  if (response.status >= 500) {
    return new AccountsError("SERVER", response.status, problemCode);
  }

  return new AccountsError("UNKNOWN", response.status, problemCode);
}

async function request(
  pathname: string,
  init: RequestInit = {},
): Promise<Response> {
  const authorization = authService.getAuthorizationHeader();

  if (authorization === null) {
    throw new AccountsError("UNAUTHORIZED", 401);
  }

  const headers = new Headers(init.headers);
  headers.set("Accept", "application/json, application/problem+json");
  headers.set("Authorization", authorization);

  let response: Response;

  try {
    response = await fetch(buildApiUrl(pathname), {
      ...init,
      headers,
      cache: "no-store",
    });
  } catch (error) {
    if (error instanceof DOMException && error.name === "AbortError") {
      throw error;
    }

    throw new AccountsError("NETWORK", 0);
  }

  if (!response.ok) {
    throw await toAccountsError(response);
  }

  return response;
}

export const accountsService = {
  async list(
    query: AccountListQuery = DEFAULT_ACCOUNT_LIST_QUERY,
    options: ListAccountsOptions = {},
  ): Promise<AccountListResponse> {
    const search = buildListSearch(query);
    const response = await request(`/admin/accounts?${search}`, {
      method: "GET",
      signal: options.signal,
    });

    let body: unknown;

    try {
      body = await response.json();
    } catch {
      throw new AccountsError("INVALID_RESPONSE", response.status);
    }

    return parseAccountListResponse(body);
  },

  async mutate(action: AccountMutation, accountId: string): Promise<void> {
    const endpoints: Record<AccountMutation, string> = {
      deactivate: `/admin/accounts/${encodeURIComponent(accountId)}/deactivate`,
      activate: `/admin/accounts/${encodeURIComponent(accountId)}/activate`,
      restore: `/admin/accounts/${encodeURIComponent(accountId)}/restore`,
    };

    await request(endpoints[action], {
      method: "POST",
    });
  },
};
