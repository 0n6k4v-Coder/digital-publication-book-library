import { useEffect, useId, useRef, useState } from "react";
import type { FormEvent } from "react";
import { AuthenticationError } from "../../services/auth";
import type { LoginCredentials } from "../../types/auth";

interface LoginPageProps {
  onLogin: (credentials: LoginCredentials) => Promise<void>;
}

interface LoginFieldErrors {
  email?: string;
  password?: string;
}

type LoginField = keyof LoginFieldErrors;

export function validateLoginCredentials(
  credentials: LoginCredentials,
): LoginFieldErrors {
  const errors: LoginFieldErrors = {};

  if (credentials.email.trim().length === 0) {
    errors.email = "Email is required.";
  }

  if (credentials.password.length === 0) {
    errors.password = "Password is required.";
  }

  return errors;
}

function getAuthenticationErrorMessage(error: unknown): string {
  if (!(error instanceof AuthenticationError)) {
    return "We could not sign you in. Please try again.";
  }

  switch (error.code) {
    case "INVALID_REQUEST":
      return "Please check the required fields and try again.";
    case "INVALID_CREDENTIALS":
      return "The email or password is incorrect.";
    case "AUTHENTICATION_RATE_LIMITED":
      return "Too many sign-in attempts. Please try again later.";
    default:
      return "We could not sign you in. Please try again.";
  }
}

function getFieldError(field: LoginField, value: string): string | undefined {
  if (field === "email" && value.trim().length === 0) {
    return "Email is required.";
  }

  if (field === "password" && value.length === 0) {
    return "Password is required.";
  }

  return undefined;
}

function joinDescribedBy(
  ...ids: Array<string | undefined>
): string | undefined {
  const value = ids.filter(Boolean).join(" ");
  return value.length > 0 ? value : undefined;
}

export function LoginPage({ onLogin }: LoginPageProps) {
  const emailId = useId();
  const passwordId = useId();
  const formErrorId = useId();

  const emailInputRef = useRef<HTMLInputElement>(null);
  const passwordInputRef = useRef<HTMLInputElement>(null);
  const formErrorRef = useRef<HTMLDivElement>(null);

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [fieldErrors, setFieldErrors] = useState<LoginFieldErrors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [showPassword, setShowPassword] = useState(false);

  useEffect(() => {
    if (formError !== null) {
      formErrorRef.current?.focus();
    }
  }, [formError]);

  async function handleSubmit(
    event: FormEvent<HTMLFormElement>,
  ): Promise<void> {
    event.preventDefault();

    if (isSubmitting) {
      return;
    }

    const credentials = { email, password };
    const errors = validateLoginCredentials(credentials);

    setFieldErrors(errors);
    setFormError(null);

    if (Object.keys(errors).length > 0) {
      if (errors.email !== undefined) {
        emailInputRef.current?.focus();
      } else if (errors.password !== undefined) {
        passwordInputRef.current?.focus();
      }

      return;
    }

    setIsSubmitting(true);

    try {
      await onLogin(credentials);
    } catch (error) {
      setFormError(getAuthenticationErrorMessage(error));
    } finally {
      setIsSubmitting(false);
    }
  }

  function handleEmailChange(value: string): void {
    setEmail(value);
    setFormError(null);
    setFieldErrors((current) => ({
      ...current,
      email: undefined,
    }));
  }

  function handlePasswordChange(value: string): void {
    setPassword(value);
    setFormError(null);
    setFieldErrors((current) => ({
      ...current,
      password: undefined,
    }));
  }

  function handleFieldBlur(field: LoginField): void {
    const value = field === "email" ? email : password;
    const error = getFieldError(field, value);

    setFieldErrors((current) => ({
      ...current,
      [field]: error,
    }));
  }

  return (
    <main className="login-page">
      <section className="login-card" aria-labelledby="login-title">
        <div className="login-card__brand">
          <div className="login-brand-mark" aria-hidden="true">
            <span>D</span>
            <span>P</span>
          </div>

          <div>
            <p className="eyebrow">Admin Application</p>
            <p className="login-card__product">
              Digital Publication &amp; Book Library
            </p>
          </div>
        </div>

        <div className="login-card__header">
          <h1 id="login-title">Sign in</h1>
          <p>Use your administrator credentials to continue.</p>
        </div>

        <form
          className="login-form"
          noValidate
          aria-busy={isSubmitting}
          onSubmit={handleSubmit}
        >
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
                <p className="form-alert__title">Sign-in failed</p>
                <p className="form-alert__message">{formError}</p>
              </div>
            </div>
          ) : null}

          <div className="field">
            <label htmlFor={emailId}>Email</label>

            <input
              ref={emailInputRef}
              id={emailId}
              name="email"
              type="email"
              autoComplete="username"
              autoCapitalize="none"
              inputMode="email"
              enterKeyHint="next"
              spellCheck={false}
              value={email}
              required
              aria-invalid={fieldErrors.email !== undefined}
              aria-describedby={joinDescribedBy(
                fieldErrors.email !== undefined
                  ? `${emailId}-error`
                  : undefined,
                formError !== null ? formErrorId : undefined,
              )}
              onBlur={() => handleFieldBlur("email")}
              onChange={(event) => handleEmailChange(event.target.value)}
            />

            {fieldErrors.email !== undefined ? (
              <p id={`${emailId}-error`} className="field-error">
                {fieldErrors.email}
              </p>
            ) : null}
          </div>

          <div className="field">
            <div className="field__label-row">
              <label htmlFor={passwordId}>Password</label>
            </div>

            <div className="password-field">
              <input
                ref={passwordInputRef}
                id={passwordId}
                name="password"
                type={showPassword ? "text" : "password"}
                autoComplete="current-password"
                autoCapitalize="none"
                enterKeyHint="done"
                spellCheck={false}
                value={password}
                required
                aria-invalid={fieldErrors.password !== undefined}
                aria-describedby={joinDescribedBy(
                  fieldErrors.password !== undefined
                    ? `${passwordId}-error`
                    : undefined,
                  formError !== null ? formErrorId : undefined,
                )}
                onBlur={() => handleFieldBlur("password")}
                onChange={(event) => handlePasswordChange(event.target.value)}
              />

              <button
                className="password-toggle"
                type="button"
                aria-controls={passwordId}
                aria-label={showPassword ? "Hide password" : "Show password"}
                onClick={() => setShowPassword((current) => !current)}
              >
                {showPassword ? "Hide" : "Show"}
              </button>
            </div>

            {fieldErrors.password !== undefined ? (
              <p id={`${passwordId}-error`} className="field-error">
                {fieldErrors.password}
              </p>
            ) : null}
          </div>

          <button
            className="primary-button"
            type="submit"
            disabled={isSubmitting}
          >
            {isSubmitting ? (
              <>
                <span className="button-spinner" aria-hidden="true" />
                <span>Signing in…</span>
              </>
            ) : (
              "Sign In"
            )}
          </button>

          {isSubmitting ? (
            <p className="status-message" role="status" aria-live="polite">
              Signing in…
            </p>
          ) : null}
        </form>
      </section>
    </main>
  );
}
