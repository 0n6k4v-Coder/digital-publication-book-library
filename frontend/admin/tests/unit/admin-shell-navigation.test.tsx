import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AdminShell } from "../../src/layouts/AdminShell";

describe("AdminShell navigation", () => {
  beforeEach(() => {
    window.history.replaceState({}, "", "/admin");
  });

  afterEach(() => {
    window.history.replaceState({}, "", "/admin");
  });

  it("renders Accounts navigation and activates it on /admin/accounts", async () => {
    const user = userEvent.setup();

    render(
      <AdminShell onLogout={vi.fn().mockResolvedValue(undefined)}>
        <p>Account feature content</p>
      </AdminShell>,
    );

    const accountsLink = screen.getByRole("link", { name: "Accounts" });

    expect(accountsLink).toHaveAttribute("href", "/admin/accounts");
    expect(accountsLink).not.toHaveAttribute("aria-current", "page");
    expect(
      screen.getByRole("main", { name: "Main content" }),
    ).toHaveTextContent("Account feature content");

    await user.click(accountsLink);

    await waitFor(() => {
      expect(window.location.pathname).toBe("/admin/accounts");
    });

    expect(screen.getByRole("link", { name: "Accounts" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("link", { name: "Dashboard" })).not.toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.getByRole("heading", {
        name: "Digital Publication & Book Library",
      }),
    ).toBeInTheDocument();
  });

  it("opens and dismisses mobile navigation without changing the route", async () => {
    const user = userEvent.setup();

    render(
      <AdminShell onLogout={vi.fn().mockResolvedValue(undefined)}>
        <p>Account feature content</p>
      </AdminShell>,
    );

    const menuButton = screen.getByRole("button", {
      name: "Open admin navigation",
    });

    await user.click(menuButton);

    expect(
      screen.getByRole("button", { name: "Close admin navigation" }),
    ).toBeInTheDocument();

    expect(menuButton).toHaveAttribute("aria-expanded", "true");

    await user.keyboard("{Escape}");

    await waitFor(() => {
      expect(menuButton).toHaveAttribute("aria-expanded", "false");
    });

    expect(window.location.pathname).toBe("/admin");
    expect(menuButton).toHaveFocus();
  });

  it("supports closing mobile navigation through the dismiss control", async () => {
    const user = userEvent.setup();

    render(
      <AdminShell onLogout={vi.fn().mockResolvedValue(undefined)}>
        <p>Account feature content</p>
      </AdminShell>,
    );

    await user.click(
      screen.getByRole("button", { name: "Open admin navigation" }),
    );

    await user.click(
      screen.getByRole("button", { name: "Dismiss admin navigation" }),
    );

    expect(window.location.pathname).toBe("/admin");
    expect(
      screen.getByRole("button", { name: "Open admin navigation" }),
    ).toHaveFocus();
  });
});
