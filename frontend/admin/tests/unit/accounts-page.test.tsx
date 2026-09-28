import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AccountsError, accountsService } from "../../src/services/accounts";
import { AccountsPage } from "../../src/pages/accounts/AccountsPage";

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
      list: vi.fn(),
      mutate: vi.fn(),
    },
    AccountsError: MockAccountsError,
  };
});

const mockedList = vi.mocked(accountsService.list);

const activeAccount = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  displayName: "Library Administrator",
  status: "active" as const,
  createdAt: "2026-09-23T10:00:00Z",
  updatedAt: "2026-09-23T10:00:00Z",
  deletedAt: null,
};

const deletedAccount = {
  ...activeAccount,
  id: "01900000-0000-7000-8000-000000000003",
  deletedAt: "2026-09-24T10:00:00Z",
  status: "inactive" as const,
};

beforeEach(() => {
  window.history.replaceState({}, "", "/admin/accounts");

  mockedList.mockReset();
});

afterEach(() => {
  vi.clearAllMocks();
});

describe("AccountsPage", () => {
  it("renders loading state and then the account list", async () => {
    mockedList.mockResolvedValue({
      items: [activeAccount],
      page: 1,
      pageSize: 20,
      total: 1,
    });

    render(<AccountsPage />);

    expect(screen.getByRole("status")).toHaveTextContent(
      "Loading administrator accounts…",
    );

    expect(
      await screen.findByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeInTheDocument();

    expect(screen.getByRole("table")).toBeInTheDocument();

    expect(
      screen.getByRole("columnheader", { name: "Email" }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("cell", { name: "admin@example.com" }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: "View account: Library Administrator",
      }),
    ).toHaveAttribute("href", `/admin/accounts/${activeAccount.id}`);

    expect(
      screen.getByRole("link", {
        name: "Edit account: Library Administrator",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${activeAccount.id}/edit?return_to=%2Fadmin%2Faccounts`,
    );

    expect(
      screen.getByRole("link", {
        name: "Delete account: Library Administrator",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${activeAccount.id}/delete?return_to=%2Fadmin%2Faccounts`,
    );

    expect(
      screen.getByRole("button", {
        name: "Deactivate account: Library Administrator",
      }),
    ).toBeInTheDocument();
  });

  it("preserves filtered Account List state in Delete navigation", async () => {
    window.history.replaceState(
      {},
      "",
      "/admin/accounts?page=2&page_size=50&status=inactive&include_deleted=true",
    );

    mockedList.mockResolvedValue({
      items: [activeAccount],
      page: 2,
      pageSize: 50,
      total: 51,
    });

    render(<AccountsPage />);

    await screen.findByRole("heading", {
      name: "Administrator Accounts",
    });

    expect(
      screen.getByRole("link", {
        name: "Delete account: Library Administrator",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${activeAccount.id}/delete?return_to=%2Fadmin%2Faccounts%3Fpage%3D2%26page_size%3D50%26status%3Dinactive%26include_deleted%3Dtrue`,
    );
  });

  it("renders Restore and Permanently Delete only for soft-deleted accounts", async () => {
    window.history.replaceState({}, "", "/admin/accounts?include_deleted=true");

    mockedList.mockResolvedValue({
      items: [deletedAccount],
      page: 1,
      pageSize: 20,
      total: 1,
    });

    render(<AccountsPage />);

    await screen.findByRole("heading", {
      name: "Administrator Accounts",
    });

    expect(
      screen.getByRole("button", {
        name: "Restore account: Library Administrator",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: "Permanently delete account: Library Administrator",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${deletedAccount.id}/delete?return_to=%2Fadmin%2Faccounts%3Finclude_deleted%3Dtrue`,
    );

    expect(
      screen.queryByRole("link", {
        name: "View account: Library Administrator",
      }),
    ).not.toBeInTheDocument();

    expect(
      screen.queryByRole("link", {
        name: "Edit account: Library Administrator",
      }),
    ).not.toBeInTheDocument();
  });

  it("renders backend authorization failure without inventing a client permission rule", async () => {
    mockedList.mockRejectedValue(new AccountsError("FORBIDDEN", 403));

    render(<AccountsPage />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "You do not have permission to view administrator accounts.",
    );
  });
});
