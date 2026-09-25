import { authService } from "./auth";
import type {
  AccountListResponse,
  AccountStatus,
  AdministratorAccount,
} from "../types/account";

interface ListAccountsOptions {
  signal?: AbortSignal;
}

type AccountsErrorCode =
  | "UNAUTHORIZED"
  | "FORBIDDEN"
  | "INVALID_RESPONSE"
  | "NETWORK"
  | "UNKNOWN";

const apiOrigin = (import.meta.env.VITE_API_ORIGIN ?? "")
  .trim()
  .replace(/\/$/, "");

export class AccountsError extends Error {
  readonly code: AccountsErrorCode;
  readonly status: number;

  constructor(code: AccountsErrorCode, status: number) {
    super(code);
    this.name = "AccountsError";
    this.code = code;
    this.status = status;
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

export const accountsService = {
  async list(options: ListAccountsOptions = {}): Promise<AccountListResponse> {
    const authorization = authService.getAuthorizationHeader();

    if (authorization === null) {
      throw new AccountsError("UNAUTHORIZED", 401);
    }

    let response: Response;

    try {
      response = await fetch(
        buildApiUrl("/admin/accounts?page=1&page_size=20"),
        {
          method: "GET",
          headers: {
            Accept: "application/json",
            Authorization: authorization,
          },
          cache: "no-store",
          signal: options.signal,
        },
      );
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") {
        throw error;
      }

      throw new AccountsError("NETWORK", 0);
    }

    if (response.status === 401) {
      authService.clearClientState();
      throw new AccountsError("UNAUTHORIZED", 401);
    }

    if (response.status === 403) {
      throw new AccountsError("FORBIDDEN", 403);
    }

    if (!response.ok) {
      throw new AccountsError("UNKNOWN", response.status);
    }

    let body: unknown;

    try {
      body = await response.json();
    } catch {
      throw new AccountsError("INVALID_RESPONSE", response.status);
    }

    return parseAccountListResponse(body);
  },
};