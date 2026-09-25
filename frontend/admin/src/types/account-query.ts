export type AccountStatusFilter = "active" | "inactive";

export interface AccountListQuery {
  page: number;
  pageSize: number;
  status: AccountStatusFilter | null;
  includeDeleted: boolean;
}

export const DEFAULT_ACCOUNT_LIST_QUERY: AccountListQuery = {
  page: 1,
  pageSize: 20,
  status: null,
  includeDeleted: false,
};

export const ACCOUNT_PAGE_SIZE_OPTIONS = [20, 50, 100] as const;

function parsePositiveInteger(value: string | null): number | null {
  if (value === null || !/^\d+$/.test(value)) {
    return null;
  }

  const parsed = Number(value);

  if (!Number.isSafeInteger(parsed) || parsed < 1) {
    return null;
  }

  return parsed;
}

export function parseAccountListQuery(search: string): AccountListQuery {
  const params = new URLSearchParams(search);

  const requestedPage = parsePositiveInteger(params.get("page"));
  const requestedPageSize = parsePositiveInteger(params.get("page_size"));
  const requestedStatus = params.get("status");

  const page = requestedPage ?? DEFAULT_ACCOUNT_LIST_QUERY.page;
  const pageSize =
    requestedPageSize !== null && requestedPageSize <= 100
      ? requestedPageSize
      : DEFAULT_ACCOUNT_LIST_QUERY.pageSize;
  const status =
    requestedStatus === "active" || requestedStatus === "inactive"
      ? requestedStatus
      : DEFAULT_ACCOUNT_LIST_QUERY.status;

  return {
    page,
    pageSize,
    status,
    includeDeleted: params.get("include_deleted") === "true",
  };
}

export function serializeAccountListQuery(query: AccountListQuery): string {
  const params = new URLSearchParams();

  if (query.page !== DEFAULT_ACCOUNT_LIST_QUERY.page) {
    params.set("page", String(Math.max(1, query.page)));
  }

  if (query.pageSize !== DEFAULT_ACCOUNT_LIST_QUERY.pageSize) {
    params.set("page_size", String(Math.min(100, Math.max(1, query.pageSize))));
  }

  if (query.status !== null) {
    params.set("status", query.status);
  }

  if (query.includeDeleted) {
    params.set("include_deleted", "true");
  }

  const search = params.toString();
  return search.length > 0 ? `?${search}` : "";
}
