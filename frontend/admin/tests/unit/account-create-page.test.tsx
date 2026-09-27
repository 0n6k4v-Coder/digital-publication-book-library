import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AccountCreatePage } from "../../src/pages/accounts/AccountCreatePage";
import { AccountsError, accountsService } from "../../src/services/accounts";

vi.mock("../../src/services/accounts", () => {
  class MockAccountsError extends Error {
    readonly code: string;
    readonly status: number;
    readonly problemCode: string | null;

    constructor(
      code: string,
      status: number,
      problemCode: string | null = null,
    ) {
      super(code);
      this.name = "AccountsError";
      this.code = code;
      this.status = status;
      this.problemCode = problemCode;
    }
  }

  return {
    accountsService: {
      create: vi.fn(),
    },
    AccountsError: MockAccountsError,
  };
});

const mockedCreate = vi.mocked(accountsService.create);

const createdAccount = {
  id: "01900000-0000-7000-8000-000000000010",
  email: "created@example.com",
  displayName: null,
  status: "active" as const,
  createdAt: "2026-09-27T02:00:00Z",
  updatedAt: "2026-09-27T02:00:00Z",
  deletedAt: null,
};

beforeEach(() => {
  window.history.replaceState({}, "", "/admin/accounts/create");
  mockedCreate.mockReset();
});

afterEach(() => {
  vi.clearAllMocks();
});

describe("AccountCreatePage", () => {
  it("renders the documented form contract", () => {
    render(<AccountCreatePage />);

    expect(
      screen.getByRole("heading", {
        level: 1,
        name: "Create Administrator Account",
      }),
    ).toBeInTheDocument();

    const email = screen.getByLabelText("Email");
    expect(email).toHaveAttribute("type", "email");
    expect(email).toBeRequired();
    expect(email).toHaveAttribute("autocomplete", "email");

    const password = screen.getByLabelText("Password");
    expect(password).toHaveAttribute("type", "password");
    expect(password).toBeRequired();
    expect(password).toHaveAttribute("autocomplete", "new-password");
    expect(password).not.toHaveAttribute("minlength");

    expect(screen.queryByLabelText("Display name")).not.toBeInTheDocument();
    expect(
      screen.queryByLabelText("Password confirmation"),
    ).not.toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", "/admin/accounts");

    expect(
      screen.getByRole("link", {
        name: "Cancel",
      }),
    ).toHaveAttribute("href", "/admin/accounts");
  });

  it("uses native required and email validation before creating an account", () => {
    render(<AccountCreatePage />);

    const form = screen.getByRole("form", {
      name: "Account credentials",
    }) as HTMLFormElement;

    const email = screen.getByLabelText("Email");
    const password = screen.getByLabelText("Password");

    expect(form.checkValidity()).toBe(false);

    fireEvent.submit(form);

    expect(mockedCreate).not.toHaveBeenCalled();
    expect(email).toBeInvalid();
    expect(password).toBeInvalid();

    fireEvent.change(email, {
      target: { value: "not-an-email" },
    });

    expect(form.checkValidity()).toBe(false);
  });

  it("does not enforce an unsupported frontend password policy", async () => {
    mockedCreate.mockResolvedValue(createdAccount);

    render(<AccountCreatePage />);

    fireEvent.change(screen.getByLabelText("Email"), {
      target: { value: "created@example.com" },
    });

    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "x" },
    });

    fireEvent.submit(
      screen.getByRole("form", {
        name: "Account credentials",
      }),
    );

    await waitFor(() => {
      expect(mockedCreate).toHaveBeenCalledWith({
        email: "created@example.com",
        password: "x",
      });
    });
  });

  it("prevents duplicate submission and disables the create action while pending", async () => {
    let resolveCreate!: (value: typeof createdAccount) => void;

    mockedCreate.mockImplementation(
      () =>
        new Promise<typeof createdAccount>((resolve) => {
          resolveCreate = resolve;
        }),
    );

    render(<AccountCreatePage />);

    fireEvent.change(screen.getByLabelText("Email"), {
      target: { value: "created@example.com" },
    });

    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "example-secure-password" },
    });

    const form = screen.getByRole("form", {
      name: "Account credentials",
    });

    fireEvent.submit(form);
    fireEvent.submit(form);

    expect(mockedCreate).toHaveBeenCalledTimes(1);

    const submitButton = screen.getByRole("button", {
      name: "Creating…",
    });

    expect(submitButton).toBeDisabled();

    expect(
      screen.getByRole("status", {
        name: "",
      }),
    ).toHaveTextContent("Creating administrator account…");

    resolveCreate(createdAccount);

    await waitFor(() => {
      expect(
        screen.getByRole("button", {
          name: "Create Account",
        }),
      ).toBeEnabled();
    });
  });

  it("renders EMAIL_ALREADY_IN_USE as an Email field error and preserves retry state", async () => {
    mockedCreate.mockRejectedValue(
      new AccountsError("CONFLICT", 409, "EMAIL_ALREADY_IN_USE"),
    );

    render(<AccountCreatePage />);

    fireEvent.change(screen.getByLabelText("Email"), {
      target: { value: "existing@example.com" },
    });

    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "example-secure-password" },
    });

    fireEvent.submit(
      screen.getByRole("form", {
        name: "Account credentials",
      }),
    );

    const error = await screen.findByText(
      "This email address is already in use.",
    );

    const email = screen.getByLabelText("Email");

    expect(error).toBeInTheDocument();
    expect(email).toHaveAttribute("aria-invalid", "true");

    const describedBy = email.getAttribute("aria-describedby");
    expect(describedBy).not.toBeNull();

    const describedError = document.getElementById(describedBy ?? "");
    expect(describedError).toHaveTextContent(
      "This email address is already in use.",
    );

    expect(email).toHaveValue("existing@example.com");
    expect(screen.getByLabelText("Password")).toHaveValue(
      "example-secure-password",
    );

    await waitFor(() => {
      expect(email).toHaveFocus();
    });

    expect(
      screen.getByRole("button", {
        name: "Create Account",
      }),
    ).toBeEnabled();

    expect(window.location.pathname).toBe("/admin/accounts/create");
  });

  it("renders server VALIDATION_ERROR without exposing server diagnostics", async () => {
    mockedCreate.mockRejectedValue(
      new AccountsError("VALIDATION", 422, "VALIDATION_ERROR"),
    );

    render(<AccountCreatePage />);

    fireEvent.change(screen.getByLabelText("Email"), {
      target: { value: "created@example.com" },
    });

    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "example-secure-password" },
    });

    fireEvent.submit(
      screen.getByRole("form", {
        name: "Account credentials",
      }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The server rejected the account details. Review the fields and try again.",
    );

    expect(
      screen.queryByText(/stack|trace|database|internal/i),
    ).not.toBeInTheDocument();

    expect(screen.getByLabelText("Email")).toHaveAttribute(
      "aria-describedby",
      expect.stringMatching(/.+/),
    );

    expect(screen.getByLabelText("Password")).toHaveAttribute(
      "aria-describedby",
      expect.stringMatching(/.+/),
    );
  });

  it("keeps authenticated state on a 403 authorization failure", async () => {
    mockedCreate.mockRejectedValue(
      new AccountsError("FORBIDDEN", 403, "ACCOUNT_CREATE_FORBIDDEN"),
    );

    render(<AccountCreatePage />);

    fireEvent.change(screen.getByLabelText("Email"), {
      target: { value: "created@example.com" },
    });

    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "example-secure-password" },
    });

    fireEvent.submit(
      screen.getByRole("form", {
        name: "Account credentials",
      }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "You do not have permission to create administrator accounts.",
    );

    expect(window.location.pathname).toBe("/admin/accounts/create");
  });

  it("clears transient form state before navigating to the created account", async () => {
    mockedCreate.mockResolvedValue(createdAccount);

    render(<AccountCreatePage />);

    fireEvent.change(screen.getByLabelText("Email"), {
      target: { value: "created@example.com" },
    });

    fireEvent.change(screen.getByLabelText("Password"), {
      target: { value: "example-secure-password" },
    });

    fireEvent.submit(
      screen.getByRole("form", {
        name: "Account credentials",
      }),
    );

    await waitFor(() => {
      expect(window.location.pathname).toBe(
        `/admin/accounts/${createdAccount.id}`,
      );
    });

    expect(screen.getByLabelText("Email")).toHaveValue("");
    expect(screen.getByLabelText("Password")).toHaveValue("");
  });
});
