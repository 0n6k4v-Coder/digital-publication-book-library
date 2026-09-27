import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountsError, accountsService } from "../../src/services/accounts";
import { AccountDetailPage } from "../../src/pages/accounts/AccountDetailPage";

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
    },
  };
});

const mockedGet = vi.mocked(accountsService.get);

const activeAccount = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  displayName: "Library Administrator",
  status: "active" as const,
  createdAt: "2026-09-23T10:00:00Z",
  updatedAt: "2026-09-23T10:00:00Z",
  deletedAt: null,
};

beforeEach(() => {
  window.history.replaceState({}, "", `/admin/accounts/${activeAccount.id}`);

  mockedGet.mockReset();
  mockedGet.mockResolvedValue(activeAccount);
});

describe("AccountDetailPage", () => {
  it("renders server-authoritative account data as read-only content", async () => {
    render(<AccountDetailPage />);

    expect(screen.getByRole("status")).toHaveTextContent(
      "Loading administrator account…",
    );

    expect(
      await screen.findByRole("heading", {
        name: "Administrator Account",
        exact: true,
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByText("Library Administrator", {
        selector: "dd",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("admin@example.com", {
        selector: "dd",
      }),
    ).toBeInTheDocument();
    expect(screen.getAllByText(activeAccount.id)).toHaveLength(2);

    expect(
      screen.getByText("Active", {
        selector: ".account-detail-status",
      }),
    ).toBeInTheDocument();

    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", {
        name: /save/i,
      }),
    ).not.toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: /deactivate|activate|restore/i,
      }),
    ).not.toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", "/admin/accounts");

    expect(
      screen.getByRole("link", {
        name: "Edit",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${activeAccount.id}/edit?return_to=%2Fadmin%2Faccounts`,
    );

    expect(mockedGet).toHaveBeenCalledWith(
      activeAccount.id,
      expect.objectContaining({
        signal: expect.any(AbortSignal),
      }),
    );
  });

  it("does not expose password data", async () => {
    render(<AccountDetailPage />);

    await screen.findByRole("heading", {
      name: "Administrator Account",
      exact: true,
    });

    expect(screen.queryByLabelText(/password/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/password_hash/i)).not.toBeInTheDocument();
  });

  it("renders a not-found error without mutation controls", async () => {
    mockedGet.mockRejectedValueOnce(
      new AccountsError("NOT_FOUND", 404, "ACCOUNT_NOT_FOUND"),
    );

    render(<AccountDetailPage />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The requested administrator account could not be found.",
    );

    expect(
      screen.queryByRole("link", {
        name: "Edit",
      }),
    ).not.toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: /delete|activate|deactivate|restore|save/i,
      }),
    ).not.toBeInTheDocument();
  });
});
