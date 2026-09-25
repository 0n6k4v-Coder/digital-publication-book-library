import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AuthenticationError } from "../../src/services/auth";
import {
  LoginPage,
  validateLoginCredentials,
} from "../../src/pages/login/LoginPage";

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("validateLoginCredentials", () => {
  it("requires both fields and does not impose a password policy", () => {
    expect(validateLoginCredentials({ email: "", password: "" })).toEqual({
      email: "Email is required.",
      password: "Password is required.",
    });

    expect(
      validateLoginCredentials({ email: "admin@example.com", password: "x" }),
    ).toEqual({});
  });
});

describe("LoginPage", () => {
  it("renders accessible email, password, and sign-in controls", () => {
    render(<LoginPage onLogin={vi.fn().mockResolvedValue(undefined)} />);

    expect(
      screen.getByRole("heading", { name: "Sign in" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Email")).toHaveAttribute("type", "email");
    expect(screen.getByLabelText("Password")).toHaveAttribute(
      "type",
      "password",
    );
    expect(screen.getByRole("button", { name: "Sign In" })).toBeEnabled();
  });

  it("shows required-field errors without submitting", async () => {
    const user = userEvent.setup();
    const onLogin = vi.fn().mockResolvedValue(undefined);

    render(<LoginPage onLogin={onLogin} />);

    await user.click(screen.getByRole("button", { name: "Sign In" }));

    expect(screen.getByText("Email is required.")).toBeInTheDocument();
    expect(screen.getByText("Password is required.")).toBeInTheDocument();
    expect(onLogin).not.toHaveBeenCalled();
  });

  it("disables submission and announces loading while authentication is pending", async () => {
    const user = userEvent.setup();
    let resolveLogin!: () => void;
    const onLogin = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          resolveLogin = resolve;
        }),
    );

    render(<LoginPage onLogin={onLogin} />);

    await user.type(screen.getByLabelText("Email"), "admin@example.com");
    await user.type(
      screen.getByLabelText("Password"),
      "example-secure-password",
    );
    await user.click(screen.getByRole("button", { name: "Sign In" }));

    expect(screen.getByRole("button", { name: "Signing in…" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("Signing in…");

    resolveLogin();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Sign In" })).toBeEnabled(),
    );
  });

  it.each([
    ["INVALID_REQUEST", "Please check the required fields and try again."],
    ["INVALID_CREDENTIALS", "The email or password is incorrect."],
    [
      "AUTHENTICATION_RATE_LIMITED",
      "Too many sign-in attempts. Please try again later.",
    ],
  ] as const)("renders a generic %s error", async (code, expectedMessage) => {
    const user = userEvent.setup();
    const status =
      code === "INVALID_REQUEST"
        ? 400
        : code === "INVALID_CREDENTIALS"
          ? 401
          : 429;
    const onLogin = vi
      .fn()
      .mockRejectedValue(new AuthenticationError(code, status));

    render(<LoginPage onLogin={onLogin} />);

    await user.type(screen.getByLabelText("Email"), "admin@example.com");
    await user.type(
      screen.getByLabelText("Password"),
      "example-secure-password",
    );
    await user.click(screen.getByRole("button", { name: "Sign In" }));

    expect(screen.getByRole("alert")).toHaveTextContent(expectedMessage);
  });
});
