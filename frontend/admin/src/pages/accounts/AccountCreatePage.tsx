import { useEffect, useId, useRef, useState } from "react";
import type { FormEvent, MouseEvent } from "react";
import { AccountsError, accountsService } from "../../services/accounts";
import { navigateTo } from "../../services/navigation";
import { buildAccountDetailHref } from "./accountRoutes";
import "./accounts.css";

function getValidationErrorMessage(error: AccountsError): string {
  if (error.problemCode === "INVALID_REQUEST") {
    return "The submitted account details could not be processed. Review the fields and try again.";
  }

  return "The server rejected the account details. Review the fields and try again.";
}

function getFormErrorMessage(error: AccountsError): string {
  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to create administrator accounts.";
    case "CONFLICT":
      return "The account could not be created because it conflicts with the current account state. Please review the form and try again.";
    case "NETWORK":
      return "We could not reach the account service. Please check your connection and try again.";
    case "SERVER":
      return "Something went wrong while creating the administrator account. Please try again.";
    case "UNSUPPORTED_MEDIA_TYPE":
      return "The account service rejected the request format. Please try again.";
    case "INVALID_RESPONSE":
      return "The account service returned an invalid response. Please try again.";
    case "VALIDATION":
      return getValidationErrorMessage(error);
    case "UNKNOWN":
    default:
      return "Something went wrong while creating the administrator account. Please try again.";
  }
}

export function AccountCreatePage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const [emailError, setEmailError] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const [isSubmitting, setIsSubmitting] = useState(false);

  const emailInputRef = useRef<HTMLInputElement>(null);
  const validationErrorRef = useRef<HTMLDivElement>(null);
  const formErrorRef = useRef<HTMLDivElement>(null);
  const submissionInFlightRef = useRef(false);

  const titleId = useId();
  const formTitleId = useId();
  const emailInputId = useId();
  const emailErrorId = useId();
  const passwordInputId = useId();
  const validationErrorId = useId();
  const formErrorId = useId();

  useEffect(() => {
    if (emailError !== null) {
      emailInputRef.current?.focus();
    }
  }, [emailError]);

  useEffect(() => {
    if (validationError !== null) {
      validationErrorRef.current?.focus();
    }
  }, [validationError]);

  useEffect(() => {
    if (formError !== null) {
      formErrorRef.current?.focus();
    }
  }, [formError]);

  function clearSubmissionErrors(): void {
    setEmailError(null);
    setValidationError(null);
    setFormError(null);
  }

  function handleAccountsNavigation(
    event: MouseEvent<HTMLAnchorElement>,
  ): void {
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
    navigateTo("/admin/accounts");
  }

  async function handleSubmit(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();

    if (submissionInFlightRef.current) {
      return;
    }

    const form = event.currentTarget;

    if (!form.reportValidity()) {
      return;
    }

    submissionInFlightRef.current = true;
    setIsSubmitting(true);
    clearSubmissionErrors();

    let didNavigate = false;

    try {
      const account = await accountsService.create({
        email,
        password,
      });

      setEmail("");
      setPassword("");
      clearSubmissionErrors();

      setIsSubmitting(false);
      submissionInFlightRef.current = false;

      navigateTo(buildAccountDetailHref(account.id));
      didNavigate = true;
    } catch (error: unknown) {
      if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
        return;
      }

      const normalizedError =
        error instanceof AccountsError
          ? error
          : new AccountsError("UNKNOWN", 0);

      if (normalizedError.problemCode === "EMAIL_ALREADY_IN_USE") {
        setEmailError("This email address is already in use.");
        return;
      }

      if (normalizedError.code === "VALIDATION") {
        setValidationError(getValidationErrorMessage(normalizedError));
        return;
      }

      setFormError(getFormErrorMessage(normalizedError));
    } finally {
      if (!didNavigate) {
        submissionInFlightRef.current = false;
        setIsSubmitting(false);
      }
    }
  }

  const emailDescribedBy =
    emailError !== null
      ? emailErrorId
      : validationError !== null
        ? validationErrorId
        : undefined;

  return (
    <section
      className="account-route-page account-create-page"
      aria-labelledby={titleId}
    >
      <a
        className="secondary-button account-route-page__back"
        href="/admin/accounts"
        onClick={handleAccountsNavigation}
      >
        Back to Administrator Accounts
      </a>

      <header className="account-route-page__header">
        <p className="eyebrow">Administration</p>
        <h1 id={titleId} className="account-route-page__title">
          Create Administrator Account
        </h1>
        <p className="account-route-page__description">
          Create a new administrator account using the current account service.
        </p>
      </header>

      <section className="account-create-card" aria-labelledby={formTitleId}>
        <div className="account-create-card__header">
          <p className="eyebrow">Administrator credentials</p>
          <h2 id={formTitleId} className="account-create-card__title">
            Account credentials
          </h2>
        </div>

        <form
          className="account-create-form"
          onSubmit={(event) => void handleSubmit(event)}
          aria-busy={isSubmitting}
        >
          <div className="field">
            <label htmlFor={emailInputId}>Email</label>

            <input
              ref={emailInputRef}
              id={emailInputId}
              name="email"
              type="email"
              required
              autoComplete="email"
              value={email}
              disabled={isSubmitting}
              aria-invalid={emailError !== null ? true : undefined}
              aria-describedby={emailDescribedBy}
              onChange={(event) => {
                setEmail(event.target.value);

                if (emailError !== null) {
                  setEmailError(null);
                }

                if (validationError !== null) {
                  setValidationError(null);
                }

                if (formError !== null) {
                  setFormError(null);
                }
              }}
            />

            {emailError !== null ? (
              <p id={emailErrorId} className="field-error">
                {emailError}
              </p>
            ) : null}
          </div>

          <div className="field">
            <label htmlFor={passwordInputId}>Password</label>

            <input
              id={passwordInputId}
              name="password"
              type="password"
              required
              autoComplete="new-password"
              value={password}
              disabled={isSubmitting}
              aria-describedby={
                validationError !== null ? validationErrorId : undefined
              }
              onChange={(event) => {
                setPassword(event.target.value);

                if (validationError !== null) {
                  setValidationError(null);
                }

                if (formError !== null) {
                  setFormError(null);
                }
              }}
            />
          </div>

          {validationError !== null ? (
            <div
              ref={validationErrorRef}
              id={validationErrorId}
              className="form-alert"
              role="alert"
              tabIndex={-1}
            >
              <span className="form-alert__icon" aria-hidden="true">
                !
              </span>

              <div>
                <p className="form-alert__title">
                  Unable to validate the account
                </p>
                <p className="form-alert__message">{validationError}</p>
              </div>
            </div>
          ) : null}

          <div className="account-create-form__actions">
            <a
              className="secondary-button account-create-form__action"
              href="/admin/accounts"
              onClick={handleAccountsNavigation}
            >
              Cancel
            </a>

            <button
              className="primary-button account-create-form__action"
              type="submit"
              disabled={isSubmitting}
            >
              {isSubmitting ? "Creating…" : "Create Account"}
            </button>
          </div>
        </form>

        {isSubmitting ? (
          <p
            className="account-create-form__status"
            role="status"
            aria-live="polite"
          >
            Creating administrator account…
          </p>
        ) : null}

        {formError !== null ? (
          <div
            ref={formErrorRef}
            id={formErrorId}
            className="form-alert"
            role="alert"
            tabIndex={-1}
          >
            <span className="form-alert__icon" aria-hidden="true">
              !
            </span>

            <div>
              <p className="form-alert__title">
                Unable to create administrator account
              </p>
              <p className="form-alert__message">{formError}</p>
            </div>
          </div>
        ) : null}
      </section>
    </section>
  );
}
