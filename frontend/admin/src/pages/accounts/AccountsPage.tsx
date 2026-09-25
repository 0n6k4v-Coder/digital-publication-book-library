import { useEffect, useRef, useState } from "react";
import { AccountsError, accountsService } from "../../services/accounts";
import type {
  AccountListResponse,
  AdministratorAccount,
} from "../../types/account";

type AccountsPageState =
  | {
      status: "loading";
    }
  | {
      status: "loaded";
      data: AccountListResponse;
    }
  | {
      status: "error";
      message: string;
    };

function getErrorMessage(error: unknown): string {
  if (!(error instanceof AccountsError)) {
    return "We could not load the administrator accounts. Please try again.";
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "Your account is not authorized to view administrator accounts.";
    case "NETWORK":
      return "We could not reach the account service. Please try again.";
    case "INVALID_RESPONSE":
      return "The account service returned an invalid response.";
    case "UNAUTHORIZED":
      return "Your authentication session is no longer valid.";
    default:
      return "We could not load the administrator accounts. Please try again.";
  }
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

function getStatusLabel(account: AdministratorAccount): string {
  return account.status === "active" ? "Active" : "Inactive";
}

export function AccountsPage() {
  const [state, setState] = useState<AccountsPageState>({
    status: "loading",
  });
  const [reloadToken, setReloadToken] = useState(0);
  const errorRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const controller = new AbortController();

    setState({ status: "loading" });

    accountsService
      .list({ signal: controller.signal })
      .then((data) => {
        if (controller.signal.aborted) {
          return;
        }

        setState({
          status: "loaded",
          data,
        });
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) {
          return;
        }

        if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
          return;
        }

        setState({
          status: "error",
          message: getErrorMessage(error),
        });
      });

    return () => controller.abort();
  }, [reloadToken]);

  useEffect(() => {
    if (state.status === "error") {
      errorRef.current?.focus();
    }
  }, [state.status]);

  const titleId = "accounts-page-title";

  return (
    <section
      className="account-page"
      aria-labelledby={titleId}
      aria-busy={state.status === "loading"}
    >
      <header className="account-page__header">
        <p className="eyebrow">Administration</p>
        <h2 id={titleId} className="account-page__title">
          Administrator Accounts
        </h2>
        <p className="account-page__description">
          View the administrator accounts currently available to the
          application.
        </p>
      </header>

      {state.status === "loading" ? (
        <p className="account-page__status" role="status" aria-live="polite">
          Loading administrator accounts…
        </p>
      ) : null}

      {state.status === "error" ? (
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
            <p className="form-alert__title">Accounts could not be loaded</p>
            <p className="form-alert__message">{state.message}</p>
            <button
              className="secondary-button account-page__retry"
              type="button"
              onClick={() => setReloadToken((value) => value + 1)}
            >
              Retry
            </button>
          </div>
        </div>
      ) : null}

      {state.status === "loaded" ? (
        <>
          <p className="account-page__summary" role="status" aria-live="polite">
            {state.data.total === 1
              ? "1 administrator account"
              : `${state.data.total} administrator accounts`}
          </p>

          {state.data.items.length === 0 ? (
            <div className="account-page__empty">
              <h3>No administrator accounts</h3>
              <p>No accounts were returned for the current query.</p>
            </div>
          ) : (
            <div className="account-table-scroll" tabIndex={0}>
              <table className="account-table">
                <caption>Administrator accounts</caption>
                <thead>
                  <tr>
                    <th scope="col">Email</th>
                    <th scope="col">Status</th>
                    <th scope="col">Created</th>
                    <th scope="col">Updated</th>
                  </tr>
                </thead>
                <tbody>
                  {state.data.items.map((account) => (
                    <tr key={account.id}>
                      <td className="account-table__email">{account.email}</td>
                      <td>
                        <span
                          className={`account-status account-status--${account.status}`}
                        >
                          {getStatusLabel(account)}
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
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </>
      ) : null}
    </section>
  );
}
