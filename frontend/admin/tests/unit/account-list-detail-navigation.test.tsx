import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountsPage } from "../../src/pages/accounts/AccountsPage";
import { accountsService } from "../../src/services/accounts";

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
      list: vi.fn(),
      mutate: vi.fn(),
    },
  };
});

const mockedList = vi.mocked(accountsService.list);

const account = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  displayName: "Library Administrator",
  status: "inactive" as const,
  createdAt: "2026-09-23T10:00:00Z",
  updatedAt: "2026-09-23T10:00:00Z",
  deletedAt: null,
};

const listUrl =
  "/admin/accounts?page=2&page_size=50&status=inactive&include_deleted=true";

beforeEach(() => {
  window.history.replaceState({}, "", listUrl);

  mockedList.mockReset();
  mockedList.mockResolvedValue({
    items: [account],
    page: 2,
    pageSize: 50,
    total: 100,
  });
});

describe("Account List to Detail navigation", () => {
  it("preserves the complete originating list query in the View link", async () => {
    render(<AccountsPage />);

    expect(
      await screen.findByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: "View account: Library Administrator",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${account.id}?return_to=${encodeURIComponent(listUrl)}`,
    );

    expect(mockedList).toHaveBeenCalledWith(
      {
        page: 2,
        pageSize: 50,
        status: "inactive",
        includeDeleted: true,
      },
      expect.objectContaining({
        signal: expect.any(AbortSignal),
      }),
    );
  });
});
