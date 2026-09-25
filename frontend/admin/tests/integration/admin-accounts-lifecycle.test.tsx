import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountsPage } from "../../src/pages/accounts/AccountsPage";
import { AccountsError, accountsService } from "../../src/services/accounts";
import type { AdministratorAccount } from "../../src/types/account";

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
const mockedMutate = vi.mocked(accountsService.mutate);

const activeAccount = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  displayName: "Library Administrator",
  status: "active" as const,
  createdAt: "2026-09-23T10:00:00Z",
  updatedAt: "2026-09-23T10:00:00Z",
  deletedAt: null,
};

const inactiveAccount = {
  id: "01900000-0000-7000-8000-000000000002",
  email: "editor@example.com",
  displayName: "Editor Account",
  status: "inactive" as const,
  createdAt: "2026-09-22T10:00:00Z",
  updatedAt: "2026-09-22T10:00:00Z",
  deletedAt: null,
};

const deletedAccount = {
  id: "01900000-0000-7000-8000-000000000003",
  email: "former@example.com",
  displayName: "Former Administrator",
  status: "inactive" as const,
  createdAt: "2026-09-21T10:00:00Z",
  updatedAt: "2026-09-21T10:00:00Z",
  deletedAt: "2026-09-24T10:00:00Z",
};

function listResponse(
  items: AdministratorAccount[] = [activeAccount, inactiveAccount],
) {
  return {
    items,
    page: 1,
    pageSize: 20,
    total: items.length,
  };
}

beforeEach(() => {
  window.history.replaceState({}, "", "/admin/accounts");
  mockedList.mockReset();
  mockedMutate.mockReset();
  mockedList.mockResolvedValue(listResponse());
});

describe("Admin Accounts lifecycle integration", () => {
  it("deactivates an active account only after confirmation and refreshes the list", async () => {
    const user = userEvent.setup();

    render(<AccountsPage />);

    expect(
      await screen.findByRole("button", {
        name: "Deactivate account: Library Administrator",
      }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", {
        name: "Deactivate account: Library Administrator",
      }),
    );

    expect(
      screen.getByRole("dialog", {
        name: "Deactivate administrator account?",
      }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Deactivate" }));

    await waitFor(() => {
      expect(mockedMutate).toHaveBeenCalledTimes(1);
      expect(mockedMutate).toHaveBeenCalledWith("deactivate", activeAccount.id);
      expect(mockedList).toHaveBeenCalledTimes(2);
    });

    expect(
      screen.getByText("Account deactivated.", {
        exact: true,
      }),
    ).toBeInTheDocument();
  });

  it("refreshes and reports the last-active-administrator conflict", async () => {
    const user = userEvent.setup();

    mockedMutate.mockRejectedValueOnce(
      new AccountsError("CONFLICT", 409, "LAST_ACTIVE_ADMINISTRATOR"),
    );

    render(<AccountsPage />);

    await screen.findByRole("button", {
      name: "Deactivate account: Library Administrator",
    });

    await user.click(
      screen.getByRole("button", {
        name: "Deactivate account: Library Administrator",
      }),
    );

    await user.click(screen.getByRole("button", { name: "Deactivate" }));

    await waitFor(() => {
      expect(mockedMutate).toHaveBeenCalledWith("deactivate", activeAccount.id);
      expect(mockedList).toHaveBeenCalledTimes(2);
    });

    expect(screen.getByRole("alert")).toHaveTextContent(
      "The last active administrator cannot be deactivated.",
    );
  });

  it("activates an inactive account and refreshes the authoritative list", async () => {
    const user = userEvent.setup();

    mockedList.mockResolvedValueOnce(listResponse([inactiveAccount]));
    mockedList.mockResolvedValueOnce(
      listResponse([
        {
          ...inactiveAccount,
          status: "active",
        },
      ]),
    );

    render(<AccountsPage />);

    await screen.findByRole("button", {
      name: "Activate account: Editor Account",
    });

    await user.click(
      screen.getByRole("button", {
        name: "Activate account: Editor Account",
      }),
    );

    await waitFor(() => {
      expect(mockedMutate).toHaveBeenCalledWith("activate", inactiveAccount.id);
      expect(mockedList).toHaveBeenCalledTimes(2);
    });

    expect(
      screen.getByText("Account activated.", {
        exact: true,
      }),
    ).toBeInTheDocument();
  });

  it("restores a deleted account only when it is included in the current list", async () => {
    const user = userEvent.setup();

    window.history.replaceState({}, "", "/admin/accounts?include_deleted=true");

    mockedList.mockResolvedValueOnce(listResponse([deletedAccount]));
    mockedList.mockResolvedValueOnce(
      listResponse([
        {
          ...deletedAccount,
          status: "inactive",
          deletedAt: null,
        },
      ]),
    );

    render(<AccountsPage />);

    await screen.findByRole("button", {
      name: "Restore account: Former Administrator",
    });

    await user.click(
      screen.getByRole("button", {
        name: "Restore account: Former Administrator",
      }),
    );

    await waitFor(() => {
      expect(mockedMutate).toHaveBeenCalledWith("restore", deletedAccount.id);
      expect(mockedList).toHaveBeenCalledTimes(2);
    });

    expect(
      screen.getByText("Account restored.", {
        exact: true,
      }),
    ).toBeInTheDocument();
  });
});
