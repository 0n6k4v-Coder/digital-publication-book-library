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

beforeEach(() => {
  mockedList.mockReset();
});

afterEach(() => {
  vi.clearAllMocks();
});

describe("AccountsPage", () => {
  it("renders loading state and then the account list", async () => {
    mockedList.mockResolvedValue({
      items: [
        {
          id: "01900000-0000-7000-8000-000000000001",
          email: "admin@example.com",
          displayName: null,
          status: "active",
          createdAt: "2026-09-23T10:00:00Z",
          updatedAt: "2026-09-23T10:00:00Z",
          deletedAt: null,
        },
      ],
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
    expect(screen.getByRole("cell", { name: "Active" })).toBeInTheDocument();

    expect(mockedList).toHaveBeenCalledTimes(1);
  });

  it("renders an empty state when no accounts are returned", async () => {
    mockedList.mockResolvedValue({
      items: [],
      page: 1,
      pageSize: 20,
      total: 0,
    });

    render(<AccountsPage />);

    expect(
      await screen.findByRole("heading", {
        name: "No administrator accounts",
      }),
    ).toBeInTheDocument();

    expect(screen.queryByRole("table")).not.toBeInTheDocument();
  });

  it("renders backend authorization failure without inventing a client permission rule", async () => {
    mockedList.mockRejectedValue(new AccountsError("FORBIDDEN", 403));

    render(<AccountsPage />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "You do not have permission to view administrator accounts.",
    );
  });
});
