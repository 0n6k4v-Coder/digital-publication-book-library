import { describe, expect, it } from "vitest";
import {
  buildAccountDetailHref,
  buildDeleteAccountHref,
  resolveAccountReturnTo,
} from "../../src/pages/accounts/accountRoutes";

describe("account delete navigation", () => {
  it("builds a same-origin delete route with return_to", () => {
    expect(
      buildDeleteAccountHref(
        "01900000-0000-7000-8000-000000000001",
        "/admin/accounts?page=2&status=active",
      ),
    ).toBe(
      "/admin/accounts/01900000-0000-7000-8000-000000000001/delete?return_to=%2Fadmin%2Faccounts%3Fpage%3D2%26status%3Dactive",
    );
  });

  it("accepts only supported Account List parameters", () => {
    window.history.replaceState({}, "", "/admin/accounts/1/delete");

    expect(
      resolveAccountReturnTo(
        "?return_to=%2Fadmin%2Faccounts%3Fpage%3D3%26page_size%3D50%26status%3Dinactive%26include_deleted%3Dtrue",
      ),
    ).toBe(
      "/admin/accounts?page=3&page_size=50&status=inactive&include_deleted=true",
    );
  });

  it("rejects unsupported query parameters and values", () => {
    window.history.replaceState({}, "", "/admin/accounts/1/delete");

    const invalidValues = [
      "/admin/accounts?foo=bar",
      "/admin/accounts?page=0",
      "/admin/accounts?page_size=101",
      "/admin/accounts?status=deleted",
      "/admin/accounts?include_deleted=false",
      "/admin/accounts?page=2&page=3",
      "/admin/accounts?status=active&status=inactive",
    ];

    for (const value of invalidValues) {
      const search = new URLSearchParams({ return_to: value }).toString();
      expect(resolveAccountReturnTo(`?${search}`)).toBe("/admin/accounts");
    }
  });

  it("rejects external, fragment, and userinfo destinations", () => {
    window.history.replaceState({}, "", "/admin/accounts/1/delete");

    const invalidValues = [
      "https://evil.example/admin/accounts",
      "//evil.example/admin/accounts",
      "/admin/accounts#fragment",
      "https://user:pass@localhost/admin/accounts",
    ];

    for (const value of invalidValues) {
      const search = new URLSearchParams({ return_to: value }).toString();
      expect(resolveAccountReturnTo(`?${search}`)).toBe("/admin/accounts");
    }
  });

  it("keeps the default Account Detail route clean", () => {
    expect(
      buildAccountDetailHref(
        "01900000-0000-7000-8000-000000000001",
        "/admin/accounts",
      ),
    ).toBe("/admin/accounts/01900000-0000-7000-8000-000000000001");
  });

  it("preserves filtered Account List state for Account Detail", () => {
    expect(
      buildAccountDetailHref(
        "01900000-0000-7000-8000-000000000001",
        "/admin/accounts?page=2&page_size=50&status=inactive&include_deleted=true",
      ),
    ).toBe(
      "/admin/accounts/01900000-0000-7000-8000-000000000001?return_to=%2Fadmin%2Faccounts%3Fpage%3D2%26page_size%3D50%26status%3Dinactive%26include_deleted%3Dtrue",
    );
  });
});
