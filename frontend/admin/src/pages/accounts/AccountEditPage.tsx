import { useEffect, useId, useRef, useState } from "react";
import type { FormEvent, MouseEvent } from "react";
import { AccountsError, accountsService } from "../../services/accounts";
import { navigateTo, usePathname } from "../../services/navigation";
import type { AdministratorAccount } from "../../types/account";
import { resolveAccountReturnTo } from "./accountRoutes";
import "./account-detail.css";

const PASSWORD_MIN_LENGTH = 15;

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

function normalizeError(error: unknown): AccountsError {
  return error instanceof AccountsError
    ? error
    : new AccountsError("UNKNOWN", 0);
}

function getLoadErrorMessage(error: AccountsError): string {
  if (error.problemCode === "INVALID_ACCOUNT_ID") {
    return "The account identifier is invalid.";
  }

  if (error.problemCode === "ACCOUNT_NOT_FOUND" || error.code === "NOT_FOUND") {
    return "This administrator account is no longer available.";
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to view this administrator account.";
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

function getDisplayNameErrorMessage(error: AccountsError): string {
  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to update this administrator account.";
    case "NOT_FOUND":
      return "This administrator account is no longer available.";
    case "UNSUPPORTED_MEDIA_TYPE":
      return "The account service rejected the update format.";
    case "VALIDATION":
      return "The display name is invalid. Review the value and try again.";
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

function getEmailErrorMessage(error: AccountsError): string {
  if (error.problemCode === "EMAIL_ALREADY_IN_USE") {
    return "That email address is already in use.";
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to change this administrator account's email address.";
    case "NOT_FOUND":
      return "This administrator account is no longer available.";
    case "VALIDATION":
      return "The email address is not valid.";
    case "NETWORK":
      return "We could not reach the account service. The email address was not changed.";
    case "SERVER":
      return "The email change could not be completed because the server failed.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "The email change could not be completed. Please try again.";
  }
}

function getPasswordErrorMessage(error: AccountsError): string {
  if (error.problemCode === "PASSWORD_POLICY_VIOLATION") {
    return "The new password does not meet the password policy.";
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to change this administrator account's password.";
    case "NOT_FOUND":
      return "This administrator account is no longer available.";
    case "VALIDATION":
      return "The password change was rejected. Review the password and try again.";
    case "NETWORK":
      return "We could not reach the account service. The password was not changed.";
    case "SERVER":
      return "The password change could not be completed because the server failed.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "The password change could not be completed. Please try again.";
  }
}

function renderNotice(notice: Notice | null) {
  if (notice === null) {
    return null;
  }

  if (notice.kind === "success") {
    return (
      <p
        className="account-detail-page__notice"
        role="status"
        aria-live="polite"
      >
        {notice.message}
      </p>
    );
  }

  return (
    <div className="form-alert" role="alert">
      <span className="form-alert__icon" aria-hidden="true">
        !
      </span>
      <p className="form-alert__message">{notice.message}</p>
    </div>
  );
}

function renderStatus(account: AdministratorAccount) {
  const label = getStatusLabel(account);
  const className = label.toLowerCase();

  return (
    <span
      className={`account-detail-status account-detail-status--${className}`}
    >
      {label}
    </span>
  );
}

export function AccountEditPage() {
  const pathname = usePathname();
  const accountId = getAccountId(pathname);
  const returnTo = resolveAccountReturnTo(window.location.search);

  const [account, setAccount] = useState<AdministratorAccount | null>(null);
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const [isLoading, setIsLoading] = useState(true);
  const [isDisplayNameSaving, setIsDisplayNameSaving] = useState(false);
  const [isEmailSaving, setIsEmailSaving] = useState(false);
  const [isPasswordSaving, setIsPasswordSaving] = useState(false);

  const [loadError, setLoadError] = useState<AccountsError | null>(null);

  const [displayNameError, setDisplayNameError] = useState<string | null>(null);
  const [emailError, setEmailError] = useState<string | null>(null);
  const [passwordError, setPasswordError] = useState<string | null>(null);

  const [displayNameNotice, setDisplayNameNotice] = useState<Notice | null>(
    null,
  );
  const [emailNotice, setEmailNotice] = useState<Notice | null>(null);
  const [passwordNotice, setPasswordNotice] = useState<Notice | null>(null);

  const loadErrorRef = useRef<HTMLDivElement>(null);
  const displayNameInputRef = useRef<HTMLInputElement>(null);
  const emailInputRef = useRef<HTMLInputElement>(null);
  const passwordInputRef = useRef<HTMLInputElement>(null);

  const displayNameInputId = useId();
  const displayNameHelpId = useId();
  const displayNameErrorId = useId();

  const emailInputId = useId();
  const emailHelpId = useId();
  const emailErrorId = useId();

  const passwordInputId = useId();
  const passwordHelpId = useId();
  const passwordErrorId = useId();

  const accountInformationHeadingId = useId();
  const emailHeadingId = useId();
  const passwordHeadingId = useId();

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
    setDisplayName("");
    setEmail("");
    setPassword("");
    setDisplayNameError(null);
    setEmailError(null);
    setPasswordError(null);
    setDisplayNameNotice(null);
    setEmailNotice(null);
    setPasswordNotice(null);

    accountsService
      .get(accountId, { signal: controller.signal })
      .then((response) => {
        if (controller.signal.aborted) {
          return;
        }

        if (response.deletedAt !== null) {
          setAccount(null);
          setLoadError(
            new AccountsError("NOT_FOUND", 404, "ACCOUNT_NOT_FOUND"),
          );
          setIsLoading(false);
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

        const normalizedError = normalizeError(error);

        if (normalizedError.code === "UNAUTHORIZED") {
          return;
        }

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

  const isMutationPending =
    isDisplayNameSaving || isEmailSaving || isPasswordSaving;

  const isBusy = isLoading || isMutationPending;

  const isDisplayNameDirty =
    account !== null &&
    normalizeDisplayName(displayName) !== account.displayName;

  const isEmailDirty = account !== null && email !== account.email;

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

  function markUnavailable(error: AccountsError): void {
    setAccount(null);
    setLoadError(error);
    setDisplayNameError(null);
    setEmailError(null);
    setPasswordError(null);
  }

  async function handleDisplayNameSubmit(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();

    if (account === null || isMutationPending || !isDisplayNameDirty) {
      return;
    }

    setIsDisplayNameSaving(true);
    setDisplayNameError(null);
    setDisplayNameNotice(null);

    try {
      const updated = await accountsService.update(account.id, {
        display_name: normalizeDisplayName(displayName),
      });

      const authoritative = updated ?? (await accountsService.get(account.id));

      if (authoritative.deletedAt !== null) {
        markUnavailable(
          new AccountsError("NOT_FOUND", 404, "ACCOUNT_NOT_FOUND"),
        );
        return;
      }

      setAccount(authoritative);
      setDisplayName(authoritative.displayName ?? "");
      setDisplayNameNotice({
        kind: "success",
        message: "Account changes saved.",
      });
    } catch (error: unknown) {
      const normalizedError = normalizeError(error);

      if (normalizedError.code === "UNAUTHORIZED") {
        return;
      }

      if (normalizedError.code === "NOT_FOUND") {
        markUnavailable(normalizedError);
        return;
      }

      setDisplayNameError(
        normalizedError.code === "VALIDATION"
          ? getDisplayNameErrorMessage(normalizedError)
          : null,
      );
      setDisplayNameNotice({
        kind: "error",
        message: getDisplayNameErrorMessage(normalizedError),
      });

      if (normalizedError.code === "VALIDATION") {
        displayNameInputRef.current?.focus();
      }
    } finally {
      setIsDisplayNameSaving(false);
    }
  }

  async function handleEmailSubmit(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();

    if (account === null || isMutationPending) {
      return;
    }

    if (email.length === 0) {
      setEmailError("Enter a new email address.");
      setEmailNotice(null);
      emailInputRef.current?.focus();
      return;
    }

    setIsEmailSaving(true);
    setEmailError(null);
    setEmailNotice(null);

    try {
      const authoritative = await accountsService.changeEmail(
        account.id,
        email,
      );

      if (authoritative.deletedAt !== null) {
        markUnavailable(
          new AccountsError("NOT_FOUND", 404, "ACCOUNT_NOT_FOUND"),
        );
        return;
      }

      setAccount(authoritative);
      setEmail("");
      setEmailNotice({
        kind: "success",
        message: "Email address changed.",
      });
    } catch (error: unknown) {
      const normalizedError = normalizeError(error);

      if (normalizedError.code === "UNAUTHORIZED") {
        return;
      }

      if (normalizedError.code === "NOT_FOUND") {
        markUnavailable(normalizedError);
        return;
      }

      const message = getEmailErrorMessage(normalizedError);

      setEmailError(
        normalizedError.code === "CONFLICT" ||
          normalizedError.code === "VALIDATION"
          ? message
          : null,
      );
      setEmailNotice({
        kind: "error",
        message,
      });

      if (
        normalizedError.code === "CONFLICT" ||
        normalizedError.code === "VALIDATION"
      ) {
        emailInputRef.current?.focus();
      }
    } finally {
      setIsEmailSaving(false);
    }
  }

  async function handlePasswordSubmit(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();

    if (account === null || isMutationPending) {
      return;
    }

    if (password.length < PASSWORD_MIN_LENGTH) {
      setPasswordError(
        `New password must be at least ${PASSWORD_MIN_LENGTH} characters.`,
      );
      setPasswordNotice(null);
      passwordInputRef.current?.focus();
      return;
    }

    setIsPasswordSaving(true);
    setPasswordError(null);
    setPasswordNotice(null);

    try {
      await accountsService.changePassword(account.id, password);

      setPassword("");
      setPasswordNotice({
        kind: "success",
        message: "Password changed successfully.",
      });
    } catch (error: unknown) {
      const normalizedError = normalizeError(error);

      if (normalizedError.code === "UNAUTHORIZED") {
        return;
      }

      if (normalizedError.code === "NOT_FOUND") {
        markUnavailable(normalizedError);
        return;
      }

      const message = getPasswordErrorMessage(normalizedError);

      setPasswordError(normalizedError.code === "VALIDATION" ? message : null);
      setPasswordNotice({
        kind: "error",
        message,
      });

      if (normalizedError.code === "VALIDATION") {
        passwordInputRef.current?.focus();
      }
    } finally {
      setIsPasswordSaving(false);
    }
  }

  return (
    <section
      className="account-detail-page"
      aria-busy={isBusy}
      aria-labelledby="account-edit-title"
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
          <h1 id="account-edit-title" className="account-detail-page__title">
            Edit Administrator Account
          </h1>

          {account !== null ? (
            <>
              <p className="account-detail-page__description">
                {account.displayName ?? account.email}
              </p>
              <p className="account-detail-page__email">{account.email}</p>
              <p className="account-detail-page__state">
                Status: {renderStatus(account)}
              </p>
            </>
          ) : null}
        </div>
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
            aria-labelledby={accountInformationHeadingId}
          >
            <div className="account-detail-section-header">
              <p className="eyebrow">Account Information</p>
              <h2 id={accountInformationHeadingId}>Administrator account</h2>
            </div>

            <form
              className="account-detail-form"
              noValidate
              onSubmit={(event) => void handleDisplayNameSubmit(event)}
              aria-busy={isDisplayNameSaving}
            >
              <div className="field">
                <label htmlFor={displayNameInputId}>Display name</label>
                <input
                  ref={displayNameInputRef}
                  id={displayNameInputId}
                  name="display_name"
                  type="text"
                  autoComplete="nickname"
                  value={displayName}
                  aria-describedby={
                    displayNameError === null
                      ? displayNameHelpId
                      : `${displayNameHelpId} ${displayNameErrorId}`
                  }
                  aria-invalid={displayNameError !== null}
                  disabled={isBusy}
                  onChange={(event) => {
                    setDisplayName(event.target.value);
                    setDisplayNameError(null);
                    setDisplayNameNotice(null);
                  }}
                />
                <p id={displayNameHelpId} className="account-detail-page__hint">
                  Leave this field empty to clear the display name.
                </p>
                {displayNameError !== null ? (
                  <p
                    id={displayNameErrorId}
                    className="account-detail-page__hint account-detail-page__hint--error"
                    role="alert"
                  >
                    {displayNameError}
                  </p>
                ) : null}
              </div>

              <div className="account-detail-form__actions">
                <button
                  className="primary-button account-detail-page__save"
                  type="submit"
                  disabled={!isDisplayNameDirty || isBusy}
                >
                  {isDisplayNameSaving ? "Saving…" : "Save changes"}
                </button>
              </div>

              {isDisplayNameSaving ? (
                <p
                  className="account-detail-page__status"
                  role="status"
                  aria-live="polite"
                >
                  Saving account changes…
                </p>
              ) : null}

              {renderNotice(displayNameNotice)}
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
                <dt>Last updated</dt>
                <dd>
                  <time dateTime={account.updatedAt}>
                    {formatDate(account.updatedAt)}
                  </time>
                </dd>
              </div>
            </dl>
          </section>

          <section
            className="account-detail-card"
            aria-labelledby={emailHeadingId}
          >
            <div className="account-detail-section-header">
              <p className="eyebrow">Change Email</p>
              <h2 id={emailHeadingId}>Administrator email address</h2>
            </div>

            <dl className="account-detail-meta account-detail-meta--primary">
              <div>
                <dt>Current email</dt>
                <dd>{account.email}</dd>
              </div>
            </dl>

            <form
              className="account-detail-form"
              noValidate
              onSubmit={(event) => void handleEmailSubmit(event)}
              aria-busy={isEmailSaving}
            >
              <div className="field">
                <label htmlFor={emailInputId}>New email</label>
                <input
                  ref={emailInputRef}
                  id={emailInputId}
                  name="email"
                  type="email"
                  autoComplete="email"
                  required
                  value={email}
                  aria-describedby={
                    emailError === null
                      ? emailHelpId
                      : `${emailHelpId} ${emailErrorId}`
                  }
                  aria-invalid={emailError !== null}
                  disabled={isBusy}
                  onChange={(event) => {
                    setEmail(event.target.value);
                    setEmailError(null);
                    setEmailNotice(null);
                  }}
                />
                <p id={emailHelpId} className="account-detail-page__hint">
                  The server validates the final email syntax and uniqueness.
                </p>
                {emailError !== null ? (
                  <p
                    id={emailErrorId}
                    className="account-detail-page__hint account-detail-page__hint--error"
                    role="alert"
                  >
                    {emailError}
                  </p>
                ) : null}
              </div>

              <div className="account-detail-form__actions">
                <button
                  className="primary-button account-detail-page__save"
                  type="submit"
                  disabled={!isEmailDirty || isBusy}
                >
                  {isEmailSaving ? "Changing email…" : "Change email"}
                </button>
              </div>

              {isEmailSaving ? (
                <p
                  className="account-detail-page__status"
                  role="status"
                  aria-live="polite"
                >
                  Changing email address…
                </p>
              ) : null}

              {renderNotice(emailNotice)}
            </form>
          </section>

          <section
            className="account-detail-card"
            aria-labelledby={passwordHeadingId}
          >
            <div className="account-detail-section-header">
              <p className="eyebrow">Change Password</p>
              <h2 id={passwordHeadingId}>Administrator password</h2>
            </div>

            <form
              className="account-detail-form"
              noValidate
              onSubmit={(event) => void handlePasswordSubmit(event)}
              aria-busy={isPasswordSaving}
            >
              <div className="field">
                <label htmlFor={passwordInputId}>New password</label>
                <input
                  ref={passwordInputRef}
                  id={passwordInputId}
                  name="password"
                  type="password"
                  autoComplete="new-password"
                  minLength={PASSWORD_MIN_LENGTH}
                  required
                  value={password}
                  aria-describedby={
                    passwordError === null
                      ? passwordHelpId
                      : `${passwordHelpId} ${passwordErrorId}`
                  }
                  aria-invalid={passwordError !== null}
                  disabled={isBusy}
                  onChange={(event) => {
                    setPassword(event.target.value);
                    setPasswordError(null);
                    setPasswordNotice(null);
                  }}
                />
                <p id={passwordHelpId} className="account-detail-page__hint">
                  Minimum length: {PASSWORD_MIN_LENGTH} characters. No
                  uppercase, lowercase, number, or symbol combination is
                  required by the frontend.
                </p>
                {passwordError !== null ? (
                  <p
                    id={passwordErrorId}
                    className="account-detail-page__hint account-detail-page__hint--error"
                    role="alert"
                  >
                    {passwordError}
                  </p>
                ) : null}
              </div>

              <div className="account-detail-form__actions">
                <button
                  className="primary-button account-detail-page__save"
                  type="submit"
                  disabled={password.length < PASSWORD_MIN_LENGTH || isBusy}
                >
                  {isPasswordSaving ? "Changing password…" : "Change password"}
                </button>
              </div>

              {isPasswordSaving ? (
                <p
                  className="account-detail-page__status"
                  role="status"
                  aria-live="polite"
                >
                  Changing password…
                </p>
              ) : null}

              {renderNotice(passwordNotice)}
            </form>
          </section>
        </>
      ) : null}
    </section>
  );
}
