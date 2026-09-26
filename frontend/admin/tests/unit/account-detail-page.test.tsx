import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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
      update: vi.fn(),
      mutate: vi.fn(),
    },
  };
});

const mockedGet = vi.mocked(accountsService.get);
const mockedUpdate = vi.mocked(accountsService.update);
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
  ...activeAccount,
  id: "01900000-0000-7000-8000-000000000002",
  email: "inactive@example.com",
  status: "inactive" as const,
};

const deletedAccount = {
  ...inactiveAccount,
  id: "01900000-0000-7000-8000-000000000003",
  deletedAt: "2026-09-24T10:00:00Z",
};

beforeEach(() => {
  window.history.replaceState(
    {},
    "",
    `/admin/accounts/${activeAccount.id}/edit`,
  );

  mockedGet.mockReset();
  mockedUpdate.mockReset();
  mockedMutate.mockReset();

  mockedGet.mockResolvedValue(activeAccount);
});

describe("AccountDetailPage", () => {
  it("loads server-authoritative data and never renders password fields", async () => {
    render(<AccountDetailPage />);

    expect(screen.getByRole("status")).toHaveTextContent(
      "Loading administrator account…",
    );

    expect(
      await screen.findByRole("heading", {
        name: "Edit Administrator Account",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByDisplayValue("Library Administrator"),
    ).toBeInTheDocument();
    expect(screen.getByText("admin@example.com")).toBeInTheDocument();
    expect(screen.getByText(activeAccount.id)).toBeInTheDocument();
    expect(screen.getByText("Active")).toBeInTheDocument();

    expect(screen.queryByLabelText(/password/i)).not.toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", "/admin/accounts");

    expect(mockedGet).toHaveBeenCalledWith(
      activeAccount.id,
      expect.objectContaining({
        signal: expect.any(AbortSignal),
      }),
    );
  });

  it("submits only display_name, prevents duplicate saving, and adopts authoritative server state", async () => {
    const user = userEvent.setup();

    let resolveUpdate: ((value: typeof activeAccount) => void) | undefined;

    mockedUpdate.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveUpdate = resolve;
        }),
    );

    render(<AccountDetailPage />);

    const input = await screen.findByRole("textbox", {
      name: "Display name",
    });

    await user.clear(input);
    await user.type(input, "Client Name");

    const saveButton = screen.getByRole("button", {
      name: "Save changes",
    });

    await user.click(saveButton);

    await waitFor(() => {
      expect(mockedUpdate).toHaveBeenCalledTimes(1);
      expect(mockedUpdate).toHaveBeenCalledWith(activeAccount.id, {
        display_name: "Client Name",
      });
    });

    expect(saveButton).toBeDisabled();

    expect(resolveUpdate).toBeDefined();

    resolveUpdate?.({
      ...activeAccount,
      displayName: "Server Canonical Name",
    });

    await waitFor(() => {
      expect(
        screen.getByDisplayValue("Server Canonical Name"),
      ).toBeInTheDocument();
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "Account changes saved.",
    );
  });

  it("shows only the lifecycle action allowed by authoritative account state", async () => {
    const user = userEvent.setup();

    mockedGet.mockResolvedValueOnce(activeAccount);

    const { unmount } = render(<AccountDetailPage />);

    await screen.findByRole("heading", {
      name: "Edit Administrator Account",
    });

    expect(
      screen.getByRole("button", {
        name: "Deactivate account",
      }),
    ).toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: "Activate account",
      }),
    ).not.toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: "Restore account",
      }),
    ).not.toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: /delete/i,
      }),
    ).not.toBeInTheDocument();

    mockedGet.mockResolvedValueOnce(inactiveAccount);
    window.history.replaceState(
      {},
      "",
      `/admin/accounts/${inactiveAccount.id}/edit`,
    );

    unmount();
    render(<AccountDetailPage />);

    await screen.findByDisplayValue("Library Administrator");

    expect(
      screen.getByRole("button", {
        name: "Activate account",
      }),
    ).toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: "Deactivate account",
      }),
    ).not.toBeInTheDocument();

    mockedGet.mockResolvedValueOnce(deletedAccount);
    window.history.replaceState(
      {},
      "",
      `/admin/accounts/${deletedAccount.id}/edit`,
    );

    await user
      .click(
        screen.getByRole("button", {
          name: "Activate account",
        }),
      )
      .catch(() => undefined);
  });

  it("handles LAST_ACTIVE_ADMINISTRATOR as an explicit conflict", async () => {
    const user = userEvent.setup();

    mockedMutate.mockRejectedValueOnce(
      new AccountsError("CONFLICT", 409, "LAST_ACTIVE_ADMINISTRATOR"),
    );

    render(<AccountDetailPage />);

    await screen.findByRole("button", {
      name: "Deactivate account",
    });

    await user.click(
      screen.getByRole("button", {
        name: "Deactivate account",
      }),
    );

    expect(
      screen.getByRole("dialog", {
        name: "Deactivate administrator account?",
      }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", {
        name: "Deactivate",
      }),
    );

    await waitFor(() => {
      expect(screen.getByRole("alert")).toHaveTextContent(
        "The last active administrator cannot be deactivated.",
      );
    });

    expect(mockedMutate).toHaveBeenCalledWith("deactivate", activeAccount.id);
  });
});
