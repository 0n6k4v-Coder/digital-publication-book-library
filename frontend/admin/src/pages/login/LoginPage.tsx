import { useId, useState } from "react";
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

export function validateLoginCredentials(credentials: LoginCredentials): LoginFieldErrors {
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

export function LoginPage({ onLogin }: LoginPageProps) {
  const emailId = useId();
  const passwordId = useId();
  const formErrorId = useId();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [fieldErrors, setFieldErrors] = useState<LoginFieldErrors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();

    if (isSubmitting) {
      return;
    }

    const credentials = { email, password };
    const errors = validateLoginCredentials(credentials);

    setFieldErrors(errors);
    setFormError(null);

    if (Object.keys(errors).length > 0) {
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
    setFieldErrors((current) => ({ ...current, email: undefined }));
  }

  function handlePasswordChange(value: string): void {
    setPassword(value);
    setFormError(null);
    setFieldErrors((current) => ({ ...current, password: undefined }));
  }

  return (
    <main className="login-page">
      <section className="login-card" aria-labelledby="login-title">
        <div className="login-card__header">
          <p className="eyebrow">Admin Application</p>
          <h1 id="login-title">Sign in</h1>
          <p>Use your administrator credentials to continue.</p>
        </div>

        <form className="login-form" noValidate aria-busy={isSubmitting} onSubmit={handleSubmit}>
          {formError !== null ? (
            <div id={formErrorId} className="form-alert" role="alert" tabIndex={-1}>
              {formError}
            </div>
          ) : null}

          <div className="field">
            <label htmlFor={emailId}>Email</label>
            <input
              id={emailId}
              name="email"
              type="email"
              autoComplete="username"
              inputMode="email"
              value={email}
              required
              aria-invalid={fieldErrors.email !== undefined}
              aria-describedby={
                fieldErrors.email !== undefined
                  ? `${emailId}-error`
                  : formError !== null
                    ? formErrorId
                    : undefined
              }
              onChange={(event) => handleEmailChange(event.target.value)}
            />
            {fieldErrors.email !== undefined ? (
              <p id={`${emailId}-error`} className="field-error">
                {fieldErrors.email}
              </p>
            ) : null}
          </div>

          <div className="field">
            <label htmlFor={passwordId}>Password</label>
            <input
              id={passwordId}
              name="password"
              type="password"
              autoComplete="current-password"
              value={password}
              required
              aria-invalid={fieldErrors.password !== undefined}
              aria-describedby={
                fieldErrors.password !== undefined
                  ? `${passwordId}-error`
                  : formError !== null
                    ? formErrorId
                    : undefined
              }
              onChange={(event) => handlePasswordChange(event.target.value)}
            />
            {fieldErrors.password !== undefined ? (
              <p id={`${passwordId}-error`} className="field-error">
                {fieldErrors.password}
              </p>
            ) : null}
          </div>

          <button className="primary-button" type="submit" disabled={isSubmitting}>
            {isSubmitting ? "Signing in…" : "Sign In"}
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