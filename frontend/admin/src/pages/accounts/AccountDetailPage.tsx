import { useEffect, useRef, useState } from "react";
import type { MouseEvent } from "react";
import { AccountsError, accountsService } from "../../services/accounts";
import { navigateTo, usePathname } from "../../services/navigation";
import type { AdministratorAccount } from "../../types/account";
import {
  buildDeleteAccountHref,
  buildEditAccountHref,
  resolveAccountReturnTo,
} from "./accountRoutes";
import "./account-detail.css";

function getAccountId(pathname: string): string | null {
  const match = pathname.match(/^\/admin\/accounts\/([^/]+)$/);

  if (match === null) {
    return null;
  }

  try {
    return decodeURIComponent(match[1]);
  } catch {
    return match[1];
  }
}

function getStatusLabel(
  account: AdministratorAccount,
): "Active" | "Inactive" | "Deleted" {
  if (account.deletedAt !== null) {
    return "Deleted";
  }

  return account.status === "active" ? "Active" : "Inactive";
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

export function AccountDetailPage() {
  const pathname = usePathname();
  const accountId = getAccountId(pathname);
  const returnTo = resolveAccountReturnTo(window.location.search);

  const [account, setAccount] = useState<AdministratorAccount | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<AccountsError | null>(null);

  const loadErrorRef = useRef<HTMLDivElement>(null);

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

    accountsService
      .get(accountId, { signal: controller.signal })
      .then((response) => {
        if (controller.signal.aborted) {
          return;
        }

        setAccount(response);
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
  }, [accountId]);

  useEffect(() => {
    if (loadError !== null) {
      loadErrorRef.current?.focus();
    }
  }, [loadError]);

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

  function handleEdit(event: MouseEvent<HTMLAnchorElement>): void {
    if (
      event.defaultPrevented ||
      event.button !== 0 ||
      event.metaKey ||
      event.ctrlKey ||
      event.shiftKey ||
      event.altKey ||
      account === null
    ) {
      return;
    }

    event.preventDefault();
    navigateTo(buildEditAccountHref(account.id, returnTo));
  }

  function handleDelete(event: MouseEvent<HTMLAnchorElement>): void {
    if (
      event.defaultPrevented ||
      event.button !== 0 ||
      event.metaKey ||
      event.ctrlKey ||
      event.shiftKey ||
      event.altKey ||
      account === null ||
      account.deletedAt !== null
    ) {
      return;
    }

    event.preventDefault();
    navigateTo(buildDeleteAccountHref(account.id, returnTo));
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
    <section
      className="account-detail-page"
      aria-busy={isLoading}
      aria-labelledby="account-detail-title"
    >
      <a
        className="secondary-button account-detail-page__back"
        href={returnTo}
        onClick={handleBack}
      >
        Back to Administrator Accounts
      </a>

      <header className="account-detail-page__header">
        <div>
          <p className="eyebrow">Administration</p>
          <h1 id="account-detail-title" className="account-detail-page__title">
            Administrator Account
          </h1>

          {account !== null ? (
            <>
              <p className="account-detail-page__description">
                {account.displayName ?? account.email}
              </p>
              <p className="account-detail-page__email">{account.email}</p>
            </>
          ) : null}
        </div>

        {account !== null ? (
          <div className="account-detail-page__header-actions">
            <a
              className="primary-button account-detail-page__edit"
              href={buildEditAccountHref(account.id, returnTo)}
              onClick={handleEdit}
            >
              Edit
            </a>

            {account.deletedAt === null ? (
              <a
                className="secondary-button account-detail-page__delete"
                href={buildDeleteAccountHref(account.id, returnTo)}
                onClick={handleDelete}
              >
                Delete Account
              </a>
            ) : null}
          </div>
        ) : null}
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
          <section
            className="account-detail-card"
            aria-labelledby="account-information-heading"
          >
            <div className="account-detail-section-header">
              <p className="eyebrow">Account Information</p>
              <h2 id="account-information-heading">Administrator account</h2>
            </div>

            <dl className="account-detail-meta account-detail-meta--primary">
              <div>
                <dt>Display name</dt>
                <dd>{account.displayName ?? "—"}</dd>
              </div>

              <div>
                <dt>Email</dt>
                <dd>{account.email}</dd>
              </div>

              <div>
                <dt>Account ID</dt>
                <dd className="account-detail-meta__id">{account.id}</dd>
              </div>
            </dl>
          </section>

          <section
            className="account-detail-card"
            aria-labelledby="account-details-heading"
          >
            <div className="account-detail-section-header">
              <p className="eyebrow">Metadata</p>
              <h2 id="account-details-heading">Account details</h2>
            </div>

            <dl className="account-detail-meta">
              <div>
                <dt>Status</dt>
                <dd>{renderStatus(account)}</dd>
              </div>

              <div>
                <dt>Account ID</dt>
                <dd className="account-detail-meta__id">{account.id}</dd>
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
          </section>

          <section
            className="account-detail-card account-detail-lifecycle"
            aria-labelledby="account-lifecycle-heading"
          >
            <div className="account-detail-section-header">
              <p className="eyebrow">Lifecycle Summary</p>
              <h2 id="account-lifecycle-heading">Current account state</h2>
            </div>

            <p className="account-detail-page__state">
              {renderStatus(account)}
            </p>
          </section>
        </>
      ) : null}
    </section>
  );
}
