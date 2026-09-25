import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import type { MouseEvent } from "react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import {
  AccountsError,
  accountsService,
  type AccountMutation,
} from "../../services/accounts";
import { navigateTo } from "../../services/navigation";
import {
  ACCOUNT_PAGE_SIZE_OPTIONS,
  parseAccountListQuery,
  type AccountListQuery,
} from "../../types/account-query";
import type {
  AccountListResponse,
  AdministratorAccount,
} from "../../types/account";
import {
  buildAccountListHref,
  buildCreateAccountHref,
  buildEditAccountHref,
} from "./accountRoutes";
import "./accounts.css";

type MutationNotice =
  | {
      kind: "success";
      message: string;
    }
  | {
      kind: "error";
      message: string;
    };

const subscribeToLocation = (listener: () => void): (() => void) => {
  window.addEventListener("popstate", listener);
  return () => window.removeEventListener("popstate", listener);
};

const getLocationSnapshot = (): string =>
  `${window.location.pathname}${window.location.search}`;

function isPlainLeftClick(event: MouseEvent<HTMLAnchorElement>): boolean {
  return (
    !event.defaultPrevented &&
    event.button === 0 &&
    !event.metaKey &&
    !event.ctrlKey &&
    !event.shiftKey &&
    !event.altKey
  );
}

function getStatusLabel(
  account: AdministratorAccount,
): "Active" | "Inactive" | "Deleted" {
  if (account.deletedAt !== null) {
    return "Deleted";
  }

  return account.status === "active" ? "Active" : "Inactive";
}

function getAccountLabel(account: AdministratorAccount): string {
  return account.displayName ?? account.email;
}

function formatDate(value: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

function getListErrorMessage(error: AccountsError): string {
  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to view administrator accounts.";
    case "NETWORK":
      return "The account list could not be loaded. Check your connection and try again.";
    case "INVALID_RESPONSE":
      return "The account service returned an invalid response.";
    case "SERVER":
      return "The account service is temporarily unavailable. Please try again.";
    case "UNSUPPORTED_MEDIA_TYPE":
    case "VALIDATION":
      return "The account list request was rejected. Please try again.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "Unable to load administrator accounts. Please try again.";
  }
}

function getMutationErrorMessage(error: AccountsError): string {
  switch (error.problemCode) {
    case "LAST_ACTIVE_ADMINISTRATOR":
      return "The last active administrator cannot be deactivated.";
    case "ACCOUNT_ALREADY_INACTIVE":
      return "That account is already inactive. The list has been refreshed.";
    case "ACCOUNT_ALREADY_ACTIVE":
      return "That account is already active. The list has been refreshed.";
    case "ACCOUNT_SOFT_DELETED":
      return "That account is deleted and cannot be activated. The list has been refreshed.";
    case "ACCOUNT_NOT_DELETED":
      return "That account is no longer deleted. The list has been refreshed.";
    case "ACCOUNT_NOT_FOUND":
      return "That account could not be found. The list has been refreshed.";
    case "VALIDATION_ERROR":
      return "The server rejected this account change. Please try again.";
    default:
      break;
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to change administrator accounts.";
    case "NOT_FOUND":
      return "That account could not be found. The list has been refreshed.";
    case "CONFLICT":
      return "The account changed before this action completed. The list has been refreshed.";
    case "VALIDATION":
      return "The server rejected this account change. Please try again.";
    case "NETWORK":
      return "We could not reach the account service. The list was not changed.";
    case "SERVER":
      return "The account change could not be completed because the server failed.";
    case "UNSUPPORTED_MEDIA_TYPE":
      return "The account service rejected the request format.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "The account change could not be completed. Please try again.";
  }
}

function getMutationSuccessMessage(action: AccountMutation): string {
  switch (action) {
    case "deactivate":
      return "Account deactivated.";
    case "activate":
      return "Account activated.";
    case "restore":
      return "Account restored.";
  }
}

function getPageSummary(data: AccountListResponse): string {
  if (data.total === 0) {
    return "0 accounts";
  }

  const start = (data.page - 1) * data.pageSize + 1;
  const end = Math.min(data.page * data.pageSize, data.total);

  return `${start}–${end} of ${data.total} ${
    data.total === 1 ? "account" : "accounts"
  }`;
}

function shouldRefreshAfterError(error: AccountsError): boolean {
  return error.code === "NOT_FOUND" || error.code === "CONFLICT";
}

export function AccountsPage() {
  const locationKey = useSyncExternalStore(
    subscribeToLocation,
    getLocationSnapshot,
    () => "/admin/accounts",
  );

  const query = useMemo(
    () => parseAccountListQuery(window.location.search),
    [locationKey],
  );

  const [data, setData] = useState<AccountListResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<AccountsError | null>(null);
  const [reloadToken, setReloadToken] = useState(0);
  const [pendingMutation, setPendingMutation] = useState<string | null>(null);
  const [mutationNotice, setMutationNotice] = useState<MutationNotice | null>(
    null,
  );
  const [confirmAccount, setConfirmAccount] =
    useState<AdministratorAccount | null>(null);

  const errorRef = useRef<HTMLDivElement>(null);
  const confirmTriggerRef = useRef<HTMLButtonElement>(null);

  const statusId = useId();
  const pageSizeId = useId();
  const includeDeletedId = useId();

  const listHref = buildAccountListHref(query);
  const createHref = buildCreateAccountHref(listHref);
  const pageCount =
    data === null ? 1 : Math.max(1, Math.ceil(data.total / data.pageSize));
  const isInitialLoading = data === null && isLoading;
  const isBusy = isLoading || pendingMutation !== null;
  const hasFilters = query.status !== null || query.includeDeleted;

  useEffect(() => {
    const canonicalHref = buildAccountListHref(query);
    const currentHref = `${window.location.pathname}${window.location.search}`;

    if (currentHref !== canonicalHref) {
      window.history.replaceState({}, "", canonicalHref);
    }
  }, [query]);

  useEffect(() => {
    const controller = new AbortController();

    setIsLoading(true);
    setLoadError(null);

    accountsService
      .list(query, { signal: controller.signal })
      .then((response) => {
        if (controller.signal.aborted) {
          return;
        }

        setData(response);
        setIsLoading(false);
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) {
          return;
        }

        if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
          return;
        }

        const normalizedError =
          error instanceof AccountsError
            ? error
            : new AccountsError("UNKNOWN", 0);

        setLoadError(normalizedError);
        setIsLoading(false);
      });

    return () => controller.abort();
  }, [
    query.page,
    query.pageSize,
    query.status,
    query.includeDeleted,
    reloadToken,
  ]);

  useEffect(() => {
    if (data === null && loadError !== null) {
      errorRef.current?.focus();
    }
  }, [data, loadError]);

  function updateQuery(nextQuery: AccountListQuery): void {
    navigateTo(buildAccountListHref(nextQuery));
  }

  function clearFilters(): void {
    updateQuery({
      ...query,
      page: 1,
      status: null,
      includeDeleted: false,
    });
  }

  function refresh(): void {
    setMutationNotice(null);
    setReloadToken((value) => value + 1);
  }

  function handleInternalLink(
    event: MouseEvent<HTMLAnchorElement>,
    href: string,
  ): void {
    if (!isPlainLeftClick(event)) {
      return;
    }

    event.preventDefault();
    navigateTo(href);
  }

  function handleDeactivateRequest(
    account: AdministratorAccount,
    event: MouseEvent<HTMLButtonElement>,
  ): void {
    if (pendingMutation !== null) {
      return;
    }

    confirmTriggerRef.current = event.currentTarget;
    setMutationNotice(null);
    setConfirmAccount(account);
  }

  async function executeMutation(
    action: AccountMutation,
    account: AdministratorAccount,
  ): Promise<void> {
    if (pendingMutation !== null) {
      return;
    }

    setPendingMutation(`${action}:${account.id}`);
    setMutationNotice(null);

    try {
      await accountsService.mutate(action, account.id);

      setMutationNotice({
        kind: "success",
        message: getMutationSuccessMessage(action),
      });

      setReloadToken((value) => value + 1);
    } catch (error: unknown) {
      if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
        return;
      }

      const normalizedError =
        error instanceof AccountsError
          ? error
          : new AccountsError("UNKNOWN", 0);

      if (shouldRefreshAfterError(normalizedError)) {
        setReloadToken((value) => value + 1);
      }

      setMutationNotice({
        kind: "error",
        message: getMutationErrorMessage(normalizedError),
      });
    } finally {
      setPendingMutation(null);
      setConfirmAccount(null);
    }
  }

  function renderRowActions(account: AdministratorAccount) {
    const label = getAccountLabel(account);
    const editHref = buildEditAccountHref(account.id, listHref);
    const isMutationPending = pendingMutation !== null;

    if (account.deletedAt !== null) {
      return (
        <button
          className="secondary-button account-row-action"
          type="button"
          disabled={isMutationPending}
          aria-label={`Restore account: ${label}`}
          onClick={() => void executeMutation("restore", account)}
        >
          Restore
        </button>
      );
    }

    const editLink = (
      <a
        className="secondary-button account-row-action"
        href={editHref}
        aria-label={`Edit account: ${label}`}
        onClick={(event) => handleInternalLink(event, editHref)}
      >
        Edit
      </a>
    );

    if (account.status === "active") {
      return (
        <>
          {editLink}
          <button
            className="secondary-button account-row-action"
            type="button"
            disabled={isMutationPending}
            aria-label={`Deactivate account: ${label}`}
            onClick={(event) => handleDeactivateRequest(account, event)}
          >
            {pendingMutation === `deactivate:${account.id}`
              ? "Deactivating…"
              : "Deactivate"}
          </button>
        </>
      );
    }

    return (
      <>
        {editLink}
        <button
          className="secondary-button account-row-action"
          type="button"
          disabled={isMutationPending}
          aria-label={`Activate account: ${label}`}
          onClick={() => void executeMutation("activate", account)}
        >
          {pendingMutation === `activate:${account.id}`
            ? "Activating…"
            : "Activate"}
        </button>
      </>
    );
  }

  const mutationPendingForConfirm =
    confirmAccount !== null &&
    pendingMutation === `deactivate:${confirmAccount.id}`;

  return (
    <section className="account-page" aria-busy={isBusy}>
      <header className="account-page__header">
        <div>
          <p className="eyebrow">Administration</p>
          <h1 id="accounts-page-title" className="account-page__title">
            Administrator Accounts
          </h1>
          <p className="account-page__description">
            View and manage the administrator accounts currently available to
            the application.
          </p>
        </div>

        <div className="account-page__header-actions">
          <a
            className="primary-button account-page__create"
            href={createHref}
            onClick={(event) => handleInternalLink(event, createHref)}
          >
            Create Account
          </a>
        </div>
      </header>

      <div className="account-controls" aria-label="Account list filters">
        <div className="account-control">
          <label htmlFor={statusId}>Status</label>
          <select
            id={statusId}
            value={query.status ?? "all"}
            disabled={isBusy}
            onChange={(event) =>
              updateQuery({
                ...query,
                page: 1,
                status:
                  event.target.value === "active" ||
                  event.target.value === "inactive"
                    ? event.target.value
                    : null,
              })
            }
          >
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="inactive">Inactive</option>
          </select>
        </div>

        <div className="account-control">
          <label htmlFor={pageSizeId}>Page size</label>
          <select
            id={pageSizeId}
            value={query.pageSize}
            disabled={isBusy}
            onChange={(event) =>
              updateQuery({
                ...query,
                page: 1,
                pageSize: Number(event.target.value),
              })
            }
          >
            {ACCOUNT_PAGE_SIZE_OPTIONS.map((pageSize) => (
              <option key={pageSize} value={pageSize}>
                {pageSize}
              </option>
            ))}
          </select>
        </div>

        <div className="account-control account-control--checkbox">
          <input
            id={includeDeletedId}
            type="checkbox"
            checked={query.includeDeleted}
            disabled={isBusy}
            onChange={(event) =>
              updateQuery({
                ...query,
                page: 1,
                includeDeleted: event.target.checked,
              })
            }
          />
          <label htmlFor={includeDeletedId}>Include deleted</label>
        </div>

        <button
          className="secondary-button account-controls__refresh"
          type="button"
          disabled={isBusy}
          aria-busy={isLoading}
          onClick={refresh}
        >
          Refresh
        </button>
      </div>

      {isLoading ? (
        <p className="account-page__status" role="status" aria-live="polite">
          {isInitialLoading
            ? "Loading administrator accounts…"
            : "Refreshing administrator accounts…"}
        </p>
      ) : null}

      {loadError !== null ? (
        <div
          ref={errorRef}
          className="form-alert account-page__error"
          role="alert"
          tabIndex={-1}
        >
          <span className="form-alert__icon" aria-hidden="true">
            !
          </span>
          <div>
            <p className="form-alert__title">
              Unable to load administrator accounts
            </p>
            <p className="form-alert__message">
              {getListErrorMessage(loadError)}
            </p>
            <button
              className="secondary-button account-page__retry"
              type="button"
              onClick={refresh}
            >
              Try Again
            </button>
          </div>
        </div>
      ) : null}

      {mutationNotice !== null ? (
        mutationNotice.kind === "success" ? (
          <p className="account-page__notice" role="status" aria-live="polite">
            {mutationNotice.message}
          </p>
        ) : (
          <div className="form-alert account-page__mutation-error" role="alert">
            <span className="form-alert__icon" aria-hidden="true">
              !
            </span>
            <p className="form-alert__message">{mutationNotice.message}</p>
          </div>
        )
      ) : null}

      {data !== null ? (
        <>
          <p className="account-page__summary" role="status" aria-live="polite">
            {getPageSummary(data)}
          </p>

          {data.items.length === 0 ? (
            <div className="account-page__empty">
              {query.includeDeleted && query.status === null ? (
                <>
                  <h2>No deleted administrator accounts</h2>
                  <p>There are no soft-deleted accounts to restore.</p>
                </>
              ) : hasFilters ? (
                <>
                  <h2>No matching administrator accounts</h2>
                  <p>No accounts match the current filters.</p>
                  <button
                    className="secondary-button account-page__empty-action"
                    type="button"
                    onClick={clearFilters}
                  >
                    Clear Filters
                  </button>
                </>
              ) : (
                <>
                  <h2>No administrator accounts</h2>
                  <p>There are no administrator accounts to display.</p>
                  <a
                    className="primary-button account-page__empty-action"
                    href={createHref}
                    onClick={(event) => handleInternalLink(event, createHref)}
                  >
                    Create Account
                  </a>
                </>
              )}
            </div>
          ) : (
            <div
              className="account-table-scroll"
              role="region"
              aria-label="Scrollable administrator accounts table"
              tabIndex={0}
            >
              <table className="account-table">
                <caption>Administrator accounts</caption>
                <thead>
                  <tr>
                    <th scope="col">Name</th>
                    <th scope="col">Email</th>
                    <th scope="col">Status</th>
                    <th scope="col">Created</th>
                    <th scope="col">Updated</th>
                    <th scope="col">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {data.items.map((account) => {
                    const statusLabel = getStatusLabel(account);
                    const statusClass = statusLabel.toLowerCase();

                    return (
                      <tr key={account.id}>
                        <td>{account.displayName ?? "—"}</td>
                        <td className="account-table__email">
                          {account.email}
                        </td>
                        <td>
                          <span
                            className={`account-status account-status--${statusClass}`}
                          >
                            {statusLabel}
                          </span>
                        </td>
                        <td>
                          <time dateTime={account.createdAt}>
                            {formatDate(account.createdAt)}
                          </time>
                        </td>
                        <td>
                          <time dateTime={account.updatedAt}>
                            {formatDate(account.updatedAt)}
                          </time>
                        </td>
                        <td>
                          <div className="account-row-actions">
                            {renderRowActions(account)}
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}

          {data.total > 0 ? (
            <nav
              className="account-pagination"
              aria-label="Account list pagination"
            >
              <button
                className="secondary-button account-pagination__button"
                type="button"
                disabled={isBusy || query.page <= 1}
                onClick={() => updateQuery({ ...query, page: query.page - 1 })}
              >
                Previous
              </button>

              <span className="account-pagination__current" aria-live="polite">
                Page {query.page} of {pageCount}
              </span>

              <button
                className="secondary-button account-pagination__button"
                type="button"
                disabled={isBusy || query.page >= pageCount}
                onClick={() => updateQuery({ ...query, page: query.page + 1 })}
              >
                Next
              </button>
            </nav>
          ) : null}
        </>
      ) : null}

      <ConfirmDialog
        open={confirmAccount !== null}
        title="Deactivate administrator account?"
        description={
          confirmAccount === null
            ? ""
            : `This will make ${getAccountLabel(
                confirmAccount,
              )} inactive. The server will confirm the final account state.`
        }
        confirmLabel="Deactivate"
        pendingLabel="Deactivating…"
        isPending={mutationPendingForConfirm}
        returnFocusRef={confirmTriggerRef}
        onCancel={() => setConfirmAccount(null)}
        onConfirm={() => {
          if (confirmAccount !== null) {
            void executeMutation("deactivate", confirmAccount);
          }
        }}
      />
    </section>
  );
}
