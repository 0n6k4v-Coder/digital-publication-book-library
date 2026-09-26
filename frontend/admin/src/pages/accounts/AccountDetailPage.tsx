import { useEffect, useId, useRef, useState } from "react";
import type { MouseEvent, FormEvent } from "react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import {
  AccountsError,
  accountsService,
  type AccountMutation,
} from "../../services/accounts";
import { navigateTo, usePathname } from "../../services/navigation";
import type { AdministratorAccount } from "../../types/account";
import { resolveAccountReturnTo } from "./accountRoutes";
import "./account-detail.css";

type Notice =
  | {
      kind: "success";
      message: string;
    }
  | {
      kind: "error";
      message: string;
    };

function getAccountId(pathname: string): string | null {
  const match = pathname.match(/^\/admin\/accounts\/([^/]+)\/edit$/);

  if (match === null) {
    return null;
  }

  try {
    return decodeURIComponent(match[1]);
  } catch {
    return match[1];
  }
}

function normalizeDisplayName(value: string): string | null {
  const normalized = value.trim();

  return normalized.length === 0 ? null : normalized;
}

function getStatusLabel(account: AdministratorAccount): string {
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

function getLoadErrorMessage(error: AccountsError): string {
  if (error.problemCode === "INVALID_ACCOUNT_ID") {
    return "The account identifier is invalid.";
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to view this administrator account.";
    case "NOT_FOUND":
      return "The requested administrator account could not be found.";
    case "NETWORK":
      return "The account could not be loaded. Check your connection and try again.";
    case "SERVER":
      return "The account service is temporarily unavailable. Please try again.";
    case "VALIDATION":
      return "The account request was rejected. Please try again.";
    case "INVALID_RESPONSE":
      return "The account service returned an invalid response.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "The administrator account could not be loaded. Please try again.";
  }
}

function getMutationErrorMessage(
  action: AccountMutation,
  error: AccountsError,
): string {
  if (error.problemCode === "LAST_ACTIVE_ADMINISTRATOR") {
    return "The last active administrator cannot be deactivated.";
  }

  switch (error.problemCode) {
    case "ACCOUNT_ALREADY_INACTIVE":
      return "That account is already inactive. The detail has been refreshed.";
    case "ACCOUNT_ALREADY_ACTIVE":
      return "That account is already active. The detail has been refreshed.";
    case "ACCOUNT_SOFT_DELETED":
      return "That account is deleted and cannot be activated. The detail has been refreshed.";
    case "ACCOUNT_NOT_DELETED":
      return "That account is no longer deleted. The detail has been refreshed.";
    case "ACCOUNT_NOT_FOUND":
      return "That account could not be found. The detail has been refreshed.";
    case "VALIDATION_ERROR":
      return `The server rejected the ${action} request. Please try again.`;
    default:
      break;
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to change this administrator account.";
    case "NOT_FOUND":
      return "That account could not be found. The detail has been refreshed.";
    case "CONFLICT":
      return "The account changed before this action completed. The detail has been refreshed.";
    case "VALIDATION":
      return `The server rejected the ${action} request. Please try again.`;
    case "NETWORK":
      return "We could not reach the account service. The account was not changed.";
    case "SERVER":
      return "The account change could not be completed because the server failed.";
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

function getUpdateErrorMessage(error: AccountsError): string {
  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to update this administrator account.";
    case "NOT_FOUND":
      return "That account could not be found. The detail will be refreshed.";
    case "UNSUPPORTED_MEDIA_TYPE":
      return "The account service rejected the update format.";
    case "VALIDATION":
      return "The server rejected the display name. Review the value and try again.";
    case "CONFLICT":
      return "The account changed before this update completed. The detail will be refreshed.";
    case "NETWORK":
      return "We could not reach the account service. The account was not changed.";
    case "SERVER":
      return "The account update could not be completed because the server failed.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "The account update could not be completed. Please try again.";
  }
}

function isRefreshWorthyError(error: AccountsError): boolean {
  return error.code === "NOT_FOUND" || error.code === "CONFLICT";
}

export function AccountDetailPage() {
  const pathname = usePathname();
  const accountId = getAccountId(pathname);
  const returnTo = resolveAccountReturnTo(window.location.search);

  const [account, setAccount] = useState<AdministratorAccount | null>(null);
  const [displayName, setDisplayName] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [pendingMutation, setPendingMutation] =
    useState<AccountMutation | null>(null);
  const [reloadToken, setReloadToken] = useState(0);
  const [loadError, setLoadError] = useState<AccountsError | null>(null);
  const [notice, setNotice] = useState<Notice | null>(null);
  const [confirmDeactivate, setConfirmDeactivate] = useState(false);

  const loadErrorRef = useRef<HTMLDivElement>(null);
  const confirmTriggerRef = useRef<HTMLButtonElement>(null);

  const displayNameInputId = useId();
  const displayNameHelpId = useId();

  useEffect(() => {
    if (accountId === null) {
      setAccount(null);
      setIsLoading(false);
      setLoadError(new AccountsError("VALIDATION", 400, "INVALID_ACCOUNT_ID"));
      return;
    }

    const controller = new AbortController();

    setIsLoading(true);
    setLoadError(null);
    setNotice(null);

    accountsService
      .get(accountId, { signal: controller.signal })
      .then((response) => {
        if (controller.signal.aborted) {
          return;
        }

        setAccount(response);
        setDisplayName(response.displayName ?? "");
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

        setAccount(null);
        setLoadError(normalizedError);
        setIsLoading(false);
      });

    return () => controller.abort();
  }, [accountId, reloadToken]);

  useEffect(() => {
    if (loadError !== null) {
      loadErrorRef.current?.focus();
    }
  }, [loadError]);

  const isDirty =
    account !== null &&
    normalizeDisplayName(displayName) !== account.displayName;

  const isBusy = isLoading || isSaving || pendingMutation !== null;

  function handleBack(event: MouseEvent<HTMLAnchorElement>): void {
    if (
      event.defaultPrevented ||
      event.button !== 0 ||
      event.metaKey ||
      event.ctrlKey ||
      event.shiftKey ||
      event.altKey
    ) {
      return;
    }

    event.preventDefault();
    navigateTo(returnTo);
  }

  async function handleSubmit(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();

    if (account === null || isSaving || pendingMutation !== null) {
      return;
    }

    const patch = {
      display_name: normalizeDisplayName(displayName),
    };

    setIsSaving(true);
    setNotice(null);

    try {
      const updated = await accountsService.update(account.id, patch);
      const authoritative = updated ?? (await accountsService.get(account.id));

      setAccount(authoritative);
      setDisplayName(authoritative.displayName ?? "");
      setNotice({
        kind: "success",
        message: "Account changes saved.",
      });
    } catch (error: unknown) {
      if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
        return;
      }

      const normalizedError =
        error instanceof AccountsError
          ? error
          : new AccountsError("UNKNOWN", 0);

      if (isRefreshWorthyError(normalizedError)) {
        setReloadToken((value) => value + 1);
      }

      setNotice({
        kind: "error",
        message: getUpdateErrorMessage(normalizedError),
      });
    } finally {
      setIsSaving(false);
    }
  }

  async function executeLifecycleMutation(
    action: AccountMutation,
  ): Promise<void> {
    if (
      account === null ||
      isSaving ||
      pendingMutation !== null ||
      action === "deactivate"
    ) {
      return;
    }

    setPendingMutation(action);
    setNotice(null);

    try {
      const updated = await accountsService.mutate(action, account.id);
      const authoritative = updated ?? (await accountsService.get(account.id));

      setAccount(authoritative);
      setDisplayName(authoritative.displayName ?? "");
      setNotice({
        kind: "success",
        message: getMutationSuccessMessage(action),
      });
    } catch (error: unknown) {
      if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
        return;
      }

      const normalizedError =
        error instanceof AccountsError
          ? error
          : new AccountsError("UNKNOWN", 0);

      if (isRefreshWorthyError(normalizedError)) {
        setReloadToken((value) => value + 1);
      }

      setNotice({
        kind: "error",
        message: getMutationErrorMessage(action, normalizedError),
      });
    } finally {
      setPendingMutation(null);
    }
  }

  async function executeDeactivate(): Promise<void> {
    if (
      account === null ||
      isSaving ||
      pendingMutation !== null ||
      account.deletedAt !== null ||
      account.status !== "active"
    ) {
      return;
    }

    setPendingMutation("deactivate");
    setNotice(null);

    try {
      const updated = await accountsService.mutate("deactivate", account.id);
      const authoritative = updated ?? (await accountsService.get(account.id));

      setAccount(authoritative);
      setDisplayName(authoritative.displayName ?? "");
      setNotice({
        kind: "success",
        message: getMutationSuccessMessage("deactivate"),
      });
    } catch (error: unknown) {
      if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
        return;
      }

      const normalizedError =
        error instanceof AccountsError
          ? error
          : new AccountsError("UNKNOWN", 0);

      if (isRefreshWorthyError(normalizedError)) {
        setReloadToken((value) => value + 1);
      }

      setNotice({
        kind: "error",
        message: getMutationErrorMessage("deactivate", normalizedError),
      });
    } finally {
      setPendingMutation(null);
      setConfirmDeactivate(false);
    }
  }

  function renderStatus(accountValue: AdministratorAccount) {
    const label = getStatusLabel(accountValue);
    const className = label.toLowerCase();

    return (
      <span
        className={`account-detail-status account-detail-status--${className}`}
      >
        {label}
      </span>
    );
  }

  return (
    <section className="account-detail-page" aria-busy={isBusy}>
      <a
        className="secondary-button account-detail-page__back"
        href={returnTo}
        onClick={handleBack}
      >
        Back to Administrator Accounts
      </a>

      <header className="account-detail-page__header">
        <p className="eyebrow">Administration</p>
        <h1 className="account-detail-page__title">
          Edit Administrator Account
        </h1>
        <p className="account-detail-page__description">
          Update the administrator account display name and manage its lifecycle
          status.
        </p>
      </header>

      {isLoading ? (
        <p
          className="account-detail-page__status"
          role="status"
          aria-live="polite"
        >
          Loading administrator account…
        </p>
      ) : null}

      {loadError !== null ? (
        <div
          ref={loadErrorRef}
          className="form-alert account-detail-page__error"
          role="alert"
          tabIndex={-1}
        >
          <span className="form-alert__icon" aria-hidden="true">
            !
          </span>
          <div>
            <p className="form-alert__title">
              Unable to load administrator account
            </p>
            <p className="form-alert__message">
              {getLoadErrorMessage(loadError)}
            </p>
          </div>
        </div>
      ) : null}

      {account !== null ? (
        <>
          <div className="account-detail-card">
            <form
              className="account-detail-form"
              onSubmit={(event) => void handleSubmit(event)}
              aria-busy={isSaving}
            >
              <div className="field">
                <label htmlFor={displayNameInputId}>Display name</label>
                <input
                  id={displayNameInputId}
                  name="display_name"
                  type="text"
                  autoComplete="nickname"
                  value={displayName}
                  aria-describedby={displayNameHelpId}
                  disabled={isBusy}
                  onChange={(event) => setDisplayName(event.target.value)}
                />
                <p id={displayNameHelpId} className="account-detail-page__hint">
                  Enter up to the server's supported display-name length. Leave
                  this field empty to clear the display name.
                </p>
              </div>

              <div className="account-detail-form__actions">
                <button
                  className="primary-button account-detail-page__save"
                  type="submit"
                  disabled={!isDirty || isBusy}
                >
                  {isSaving ? "Saving…" : "Save changes"}
                </button>
              </div>
            </form>

            <dl className="account-detail-meta">
              <div>
                <dt>Email</dt>
                <dd>{account.email}</dd>
              </div>

              <div>
                <dt>Account ID</dt>
                <dd className="account-detail-meta__id">{account.id}</dd>
              </div>

              <div>
                <dt>Status</dt>
                <dd>{renderStatus(account)}</dd>
              </div>

              <div>
                <dt>Created</dt>
                <dd>
                  <time dateTime={account.createdAt}>
                    {formatDate(account.createdAt)}
                  </time>
                </dd>
              </div>

              <div>
                <dt>Updated</dt>
                <dd>
                  <time dateTime={account.updatedAt}>
                    {formatDate(account.updatedAt)}
                  </time>
                </dd>
              </div>

              <div>
                <dt>Deleted</dt>
                <dd>
                  {account.deletedAt === null ? (
                    "Not deleted"
                  ) : (
                    <time dateTime={account.deletedAt}>
                      {formatDate(account.deletedAt)}
                    </time>
                  )}
                </dd>
              </div>
            </dl>
          </div>

          <section
            className="account-detail-lifecycle"
            aria-labelledby="account-lifecycle-heading"
          >
            <div className="account-detail-lifecycle__header">
              <div>
                <p className="eyebrow">Lifecycle</p>
                <h2 id="account-lifecycle-heading">Account status</h2>
              </div>
              <p className="account-detail-lifecycle__state">
                Current state: <strong>{getStatusLabel(account)}</strong>
              </p>
            </div>

            <div className="account-detail-lifecycle__actions">
              {account.deletedAt !== null ? (
                <button
                  className="secondary-button account-detail-page__action"
                  type="button"
                  disabled={isBusy}
                  onClick={() => void executeLifecycleMutation("restore")}
                >
                  {pendingMutation === "restore"
                    ? "Restoring…"
                    : "Restore account"}
                </button>
              ) : account.status === "active" ? (
                <button
                  ref={confirmTriggerRef}
                  className="secondary-button account-detail-page__action"
                  type="button"
                  disabled={isBusy}
                  onClick={() => setConfirmDeactivate(true)}
                >
                  {pendingMutation === "deactivate"
                    ? "Deactivating…"
                    : "Deactivate account"}
                </button>
              ) : (
                <button
                  className="secondary-button account-detail-page__action"
                  type="button"
                  disabled={isBusy}
                  onClick={() => void executeLifecycleMutation("activate")}
                >
                  {pendingMutation === "activate"
                    ? "Activating…"
                    : "Activate account"}
                </button>
              )}
            </div>
          </section>

          {isSaving ? (
            <p
              className="account-detail-page__status"
              role="status"
              aria-live="polite"
            >
              Saving account changes…
            </p>
          ) : null}

          {pendingMutation !== null ? (
            <p
              className="account-detail-page__status"
              role="status"
              aria-live="polite"
            >
              {pendingMutation === "deactivate"
                ? "Deactivating account…"
                : pendingMutation === "activate"
                  ? "Activating account…"
                  : "Restoring account…"}
            </p>
          ) : null}

          {notice !== null ? (
            notice.kind === "success" ? (
              <p
                className="account-detail-page__notice"
                role="status"
                aria-live="polite"
              >
                {notice.message}
              </p>
            ) : (
              <div className="form-alert" role="alert">
                <span className="form-alert__icon" aria-hidden="true">
                  !
                </span>
                <p className="form-alert__message">{notice.message}</p>
              </div>
            )
          ) : null}

          <ConfirmDialog
            open={confirmDeactivate}
            title="Deactivate administrator account?"
            description={`This will make ${getAccountLabel(
              account,
            )} inactive. The server will confirm the final account state.`}
            confirmLabel="Deactivate"
            pendingLabel="Deactivating…"
            isPending={pendingMutation === "deactivate"}
            returnFocusRef={confirmTriggerRef}
            onCancel={() => setConfirmDeactivate(false)}
            onConfirm={() => void executeDeactivate()}
          />
        </>
      ) : null}
    </section>
  );
}
