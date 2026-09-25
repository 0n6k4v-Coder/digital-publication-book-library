import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AdminShell } from "../../src/layouts/AdminShell";

describe("AdminShell", () => {
  it("renders the application identity, named sidebar, and empty main content area", () => {
    render(<AdminShell onLogout={vi.fn().mockResolvedValue(undefined)} />);

    expect(
      screen.getByRole("complementary", { name: "Admin application" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        name: "Digital Publication & Book Library",
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Logout" })).toBeEnabled();

    expect(
      screen.getByRole("main", { name: "Main content" }),
    ).toBeEmptyDOMElement();
  });

  it("disables logout and announces the pending state", async () => {
    const user = userEvent.setup();
    let resolveLogout!: () => void;

    const onLogout = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          resolveLogout = resolve;
        }),
    );

    render(<AdminShell onLogout={onLogout} />);

    const logoutButton = screen.getByRole("button", { name: "Logout" });

    await user.click(logoutButton);

    expect(onLogout).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "Signing out…" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("Signing out…");
    expect(
      screen.getByRole("complementary", { name: "Admin application" }),
    ).toHaveAttribute("aria-busy", "true");

    resolveLogout();

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Logout" })).toBeEnabled(),
    );

    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    expect(
      screen.getByRole("complementary", { name: "Admin application" }),
    ).toHaveAttribute("aria-busy", "false");
  });

  it("shows a retryable logout failure and allows another attempt", async () => {
    const user = userEvent.setup();

    const onLogout = vi
      .fn()
      .mockRejectedValueOnce(new Error("logout failed"))
      .mockResolvedValueOnce(undefined);

    render(<AdminShell onLogout={onLogout} />);

    await user.click(screen.getByRole("button", { name: "Logout" }));

    expect(screen.getByRole("alert")).toHaveTextContent(
      "We could not sign you out. Please try again.",
    );
    expect(screen.getByRole("button", { name: "Logout" })).toBeEnabled();

    await user.click(screen.getByRole("button", { name: "Logout" }));

    await waitFor(() => expect(onLogout).toHaveBeenCalledTimes(2));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
