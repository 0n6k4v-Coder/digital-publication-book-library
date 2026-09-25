import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AccountsPage } from "../../src/pages/accounts/AccountsPage";
import { accountsService } from "../../src/services/accounts";

vi.mock("../../src/services/accounts", () => ({
  accountsService: {
    list: vi.fn(),
  },
  AccountsError: class AccountsError extends Error {
    readonly code: string;
    readonly status: number;

    constructor(code: string, status: number) {
      super(code);
      this.code = code;
      this.status = status;
    }
  },
}));

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
    expect(screen.getByText("Active")).toBeInTheDocument();

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
    mockedList.mockRejectedValue(new Error("FORBIDDEN"));

    render(<AccountsPage />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "We could not load the administrator accounts",
    );
  });
});
