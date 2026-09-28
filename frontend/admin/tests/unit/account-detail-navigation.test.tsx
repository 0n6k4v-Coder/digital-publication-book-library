import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountDetailPage } from "../../src/pages/accounts/AccountDetailPage";
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
      get: vi.fn(),
    },
  };
});

const mockedGet = vi.mocked(accountsService.get);

const account = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  displayName: "Library Administrator",
  status: "active" as const,
  createdAt: "2026-09-23T10:00:00Z",
  updatedAt: "2026-09-23T10:00:00Z",
  deletedAt: null,
};

const listUrl =
  "/admin/accounts?page=2&page_size=50&status=active&include_deleted=true";

beforeEach(() => {
  window.history.replaceState(
    {},
    "",
    `/admin/accounts/${account.id}?return_to=${encodeURIComponent(listUrl)}`,
  );

  mockedGet.mockReset();
  mockedGet.mockResolvedValue(account);
});

describe("Account Detail navigation context", () => {
  it("preserves the originating Account List query for Back, Edit, and Delete", async () => {
    render(<AccountDetailPage />);

    expect(
      await screen.findByRole("heading", {
        name: /^Administrator Account$/,
      }),
    ).toBeInTheDocument();

    const encodedReturnTo = encodeURIComponent(listUrl);

    expect(
      screen.getByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", listUrl);

    expect(
      screen.getByRole("link", {
        name: "Edit",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${account.id}/edit?return_to=${encodedReturnTo}`,
    );

    expect(
      screen.getByRole("link", {
        name: "Delete Account",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${account.id}/delete?return_to=${encodedReturnTo}`,
    );

    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", {
        name: /save|delete|activate|deactivate|restore/i,
      }),
    ).not.toBeInTheDocument();

    expect(mockedGet).toHaveBeenCalledWith(
      account.id,
      expect.objectContaining({
        signal: expect.any(AbortSignal),
      }),
    );
  });
});
