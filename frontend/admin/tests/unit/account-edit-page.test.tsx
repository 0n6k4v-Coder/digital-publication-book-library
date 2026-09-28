import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountEditPage } from "../../src/pages/accounts/AccountEditPage";
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
    AccountsError: MockAccountsError,
    accountsService: {
      get: vi.fn(),
      update: vi.fn(),
      changeEmail: vi.fn(),
      changePassword: vi.fn(),
    },
  };
});

const mockedGet = vi.mocked(accountsService.get);
const mockedUpdate = vi.mocked(accountsService.update);
const mockedChangeEmail = vi.mocked(accountsService.changeEmail);
const mockedChangePassword = vi.mocked(accountsService.changePassword);

const account = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  displayName: "Library Administrator",
  status: "active" as const,
  createdAt: "2026-09-23T10:00:00Z",
  updatedAt: "2026-09-23T10:00:00Z",
  deletedAt: null,
};

beforeEach(() => {
  window.history.replaceState(
    {},
    "",
    `/admin/accounts/${account.id}/edit?return_to=%2Fadmin%2Faccounts%3Fpage%3D2`,
  );

  mockedGet.mockReset();
  mockedUpdate.mockReset();
  mockedChangeEmail.mockReset();
  mockedChangePassword.mockReset();

  mockedGet.mockResolvedValue(account);
  mockedUpdate.mockResolvedValue(account);
  mockedChangeEmail.mockResolvedValue(account);
  mockedChangePassword.mockResolvedValue();
});

describe("AccountEditPage", () => {
  it("loads authoritative data and exposes only Edit-page operations", async () => {
    render(<AccountEditPage />);

    expect(
      await screen.findByRole("heading", {
        name: "Edit Administrator Account",
      }),
    ).toBeInTheDocument();

    expect(screen.getByLabelText("Display name")).toHaveValue(
      "Library Administrator",
    );

    expect(screen.getByLabelText("New email")).toBeInTheDocument();

    expect(screen.getByLabelText("New password")).toHaveAttribute(
      "type",
      "password",
    );

    expect(screen.getByLabelText("New password")).toHaveAttribute(
      "autocomplete",
      "new-password",
    );

    expect(
      screen.queryByRole("button", {
        name: /deactivate|activate|restore|delete|role/i,
      }),
    ).not.toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", "/admin/accounts?page=2");

    expect(mockedGet).toHaveBeenCalledWith(
      account.id,
      expect.objectContaining({
        signal: expect.any(AbortSignal),
      }),
    );
  });

  it("clears the display name and submits null", async () => {
    const user = userEvent.setup();
    const updatedAccount = {
      ...account,
      displayName: null,
      updatedAt: "2026-09-28T10:00:00Z",
    };

    mockedUpdate.mockResolvedValueOnce(updatedAccount);

    render(<AccountEditPage />);

    const input = await screen.findByLabelText("Display name");

    await user.clear(input);

    await user.click(
      screen.getByRole("button", {
        name: "Save changes",
      }),
    );

    await waitFor(() => {
      expect(mockedUpdate).toHaveBeenCalledWith(account.id, {
        display_name: null,
      });
    });

    expect(input).toHaveValue("");
    expect(
      screen.getByRole("status", {
        name: "",
      }),
    ).toHaveTextContent("Account changes saved.");
  });

  it("uses the returned Account as authoritative after an email change", async () => {
    const user = userEvent.setup();
    const updatedAccount = {
      ...account,
      email: "canonical@example.com",
    };

    mockedChangeEmail.mockResolvedValueOnce(updatedAccount);

    render(<AccountEditPage />);

    await screen.findByRole("heading", {
      name: "Change Email",
    });

    const input = screen.getByLabelText("New email");

    await user.type(input, "canonical@example.com");

    await user.click(
      screen.getByRole("button", {
        name: "Change email",
      }),
    );

    await waitFor(() => {
      expect(mockedChangeEmail).toHaveBeenCalledWith(
        account.id,
        "canonical@example.com",
      );
    });

    expect(screen.getAllByText("canonical@example.com")).not.toHaveLength(0);

    expect(input).toHaveValue("");

    expect(
      screen.getByRole("status", {
        name: "",
      }),
    ).toHaveTextContent("Email address changed.");
  });

  it("handles EMAIL_ALREADY_IN_USE as a field error", async () => {
    const user = userEvent.setup();

    mockedChangeEmail.mockRejectedValueOnce(
      new AccountsError("CONFLICT", 409, "EMAIL_ALREADY_IN_USE"),
    );

    render(<AccountEditPage />);

    const input = await screen.findByLabelText("New email");

    await user.type(input, "taken@example.com");
    await user.click(
      screen.getByRole("button", {
        name: "Change email",
      }),
    );

    const error = await screen.findByText(
      "That email address is already in use.",
    );

    expect(input).toHaveAttribute("aria-invalid", "true");

    const describedBy = input
      .getAttribute("aria-describedby")
      ?.split(" ")
      .map((id) => document.getElementById(id)?.textContent ?? "");

    expect(
      describedBy?.some((value) =>
        value.includes("That email address is already in use."),
      ),
    ).toBe(true);

    expect(error).toBeInTheDocument();
  });

  it("enforces the 15-character password minimum without adding composition rules", async () => {
    const user = userEvent.setup();

    render(<AccountEditPage />);

    const input = await screen.findByLabelText("New password");
    const button = screen.getByRole("button", {
      name: "Change password",
    });

    await user.type(input, "12345678901234");

    expect(button).toBeDisabled();

    await user.type(input, "5");

    expect(button).toBeEnabled();

    await user.click(button);

    await waitFor(() => {
      expect(mockedChangePassword).toHaveBeenCalledWith(
        account.id,
        "123456789012345",
      );
    });

    expect(input).toHaveValue("");

    expect(
      screen.getByRole("status", {
        name: "",
      }),
    ).toHaveTextContent("Password changed successfully.");
  });

  it("handles PASSWORD_POLICY_VIOLATION without exposing password policy internals", async () => {
    const user = userEvent.setup();

    mockedChangePassword.mockRejectedValueOnce(
      new AccountsError("VALIDATION", 422, "PASSWORD_POLICY_VIOLATION"),
    );

    render(<AccountEditPage />);

    const input = await screen.findByLabelText("New password");

    await user.type(input, "123456789012345");
    await user.click(
      screen.getByRole("button", {
        name: "Change password",
      }),
    );

    expect(
      await screen.findByText(
        "The new password does not meet the password policy.",
      ),
    ).toBeInTheDocument();

    expect(input).toHaveAttribute("aria-invalid", "true");
  });

  it("treats a soft-deleted Account response as unavailable", async () => {
    mockedGet.mockResolvedValueOnce({
      ...account,
      deletedAt: "2026-09-28T10:00:00Z",
    });

    render(<AccountEditPage />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "This administrator account is no longer available.",
    );

    expect(screen.queryByLabelText("Display name")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("New email")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("New password")).not.toBeInTheDocument();
  });
});
