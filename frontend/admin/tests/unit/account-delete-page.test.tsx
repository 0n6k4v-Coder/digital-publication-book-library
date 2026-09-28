import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountDeletePage } from "../../src/pages/accounts/AccountDeletePage";
import { AccountsError, accountsService } from "../../src/services/accounts";

const account = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  displayName: "Library Administrator",
  status: "active" as const,
  createdAt: "2026-09-23T10:00:00Z",
  updatedAt: "2026-09-23T10:00:00Z",
  deletedAt: null,
};

vi.mock("../../src/services/navigation", () => ({
  navigateTo: vi.fn(),
  usePathname: () => window.location.pathname,
}));

beforeEach(() => {
  window.history.replaceState(
    {},
    "",
    `/admin/accounts/${account.id}/delete?return_to=%2Fadmin%2Faccounts%3Fpage%3D2`,
  );

  vi.restoreAllMocks();

  vi.spyOn(accountsService, "get").mockResolvedValue(account);
  vi.spyOn(accountsService, "softDelete").mockResolvedValue(undefined);
  vi.spyOn(accountsService, "hardDelete").mockResolvedValue(undefined);
});

describe("AccountDeletePage", () => {
  it("loads the authoritative account without rendering credential data", async () => {
    render(<AccountDeletePage />);

    expect(
      await screen.findByRole("heading", {
        name: "Delete Administrator Account",
      }),
    ).toBeInTheDocument();

    expect(screen.getByText("Library Administrator")).toBeInTheDocument();

    expect(screen.getByText("admin@example.com")).toBeInTheDocument();

    expect(screen.getByText(account.id)).toBeInTheDocument();

    expect(screen.queryByText(/password/i)).not.toBeInTheDocument();

    expect(screen.queryByText(/token/i)).not.toBeInTheDocument();
  });

  it("does not expose destructive controls for an invalid Account ID", () => {
    window.history.replaceState(
      {},
      "",
      "/admin/accounts/not-a-uuid/delete?return_to=%2Fadmin%2Faccounts",
    );

    render(<AccountDeletePage />);

    expect(screen.getByRole("alert")).toHaveTextContent(
      "The delete route does not contain a valid Account ID",
    );

    expect(
      screen.queryByRole("button", {
        name: /delete/i,
      }),
    ).not.toBeInTheDocument();

    expect(accountsService.get).not.toHaveBeenCalled();
  });

  it("cancels soft-delete confirmation without mutation and restores focus", async () => {
    render(<AccountDeletePage />);

    const trigger = await screen.findByRole("button", {
      name: "Soft-delete account",
    });

    trigger.focus();
    fireEvent.click(trigger);

    expect(screen.getByRole("alertdialog")).toBeInTheDocument();

    expect(document.activeElement).toBe(
      screen.getByRole("button", {
        name: "Cancel",
      }),
    );

    expect(accountsService.softDelete).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole("button", {
        name: "Cancel",
      }),
    );

    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });

    expect(document.activeElement).toBe(trigger);

    expect(accountsService.softDelete).not.toHaveBeenCalled();
  });

  it("Escape cancels confirmation without mutation", async () => {
    render(<AccountDeletePage />);

    fireEvent.click(
      await screen.findByRole("button", {
        name: "Soft-delete account",
      }),
    );

    const dialog = screen.getByRole("alertdialog");

    fireEvent.keyDown(dialog, {
      key: "Escape",
    });

    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    });

    expect(accountsService.softDelete).not.toHaveBeenCalled();
  });

  it("submits soft delete once and keeps a duplicate click from issuing a second request", async () => {
    let resolveDelete!: () => void;

    vi.mocked(accountsService.softDelete).mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveDelete = resolve;
        }),
    );

    render(<AccountDeletePage />);

    fireEvent.click(
      await screen.findByRole("button", {
        name: "Soft-delete account",
      }),
    );

    const confirm = screen.getByRole("button", {
      name: "Delete Account",
    });

    fireEvent.click(confirm);
    fireEvent.click(confirm);

    expect(accountsService.softDelete).toHaveBeenCalledTimes(1);

    expect(screen.getByRole("status")).toHaveTextContent(
      "Soft-deleting administrator account",
    );

    resolveDelete();
  });

  it("uses the hard-delete endpoint for a soft-deleted target", async () => {
    vi.spyOn(accountsService, "get").mockResolvedValueOnce({
      ...account,
      status: "inactive",
      deletedAt: "2026-09-24T10:00:00Z",
    });

    render(<AccountDeletePage />);

    expect(
      await screen.findByRole("button", {
        name: "Permanently delete account",
      }),
    ).toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: "Soft-delete account",
      }),
    ).not.toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", {
        name: "Permanently delete account",
      }),
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Permanently Delete",
      }),
    );

    await waitFor(() => {
      expect(accountsService.hardDelete).toHaveBeenCalledWith(account.id);
    });
  });

  it("keeps the confirmation usable when the server rejects with LAST_ACTIVE_ADMINISTRATOR", async () => {
    vi.mocked(accountsService.softDelete).mockRejectedValueOnce(
      new AccountsError("CONFLICT", 409, "LAST_ACTIVE_ADMINISTRATOR"),
    );

    render(<AccountDeletePage />);

    fireEvent.click(
      await screen.findByRole("button", {
        name: "Soft-delete account",
      }),
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Delete Account",
      }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The last active administrator cannot be deleted.",
    );

    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
  });

  it("keeps a 403 on the delete page and does not logout", async () => {
    vi.mocked(accountsService.softDelete).mockRejectedValueOnce(
      new AccountsError("FORBIDDEN", 403, "ACCOUNT_DELETE_FORBIDDEN"),
    );

    render(<AccountDeletePage />);

    fireEvent.click(
      await screen.findByRole("button", {
        name: "Soft-delete account",
      }),
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Delete Account",
      }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "You do not have permission to soft-delete this administrator account.",
    );

    expect(
      screen.getByRole("heading", {
        name: "Delete Administrator Account",
      }),
    ).toBeInTheDocument();
  });

  it("retains only the validated return destination after success", async () => {
    const navigation = await import("../../src/services/navigation");

    render(<AccountDeletePage />);

    fireEvent.click(
      await screen.findByRole("button", {
        name: "Soft-delete account",
      }),
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Delete Account",
      }),
    );

    await waitFor(() => {
      expect(navigation.navigateTo).toHaveBeenCalledWith(
        "/admin/accounts?page=2",
        { replace: true },
      );
    });
  });
});
