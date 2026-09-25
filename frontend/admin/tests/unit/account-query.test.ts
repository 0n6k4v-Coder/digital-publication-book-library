import { describe, expect, it } from "vitest";
import {
  DEFAULT_ACCOUNT_LIST_QUERY,
  parseAccountListQuery,
  serializeAccountListQuery,
} from "../../src/types/account-query";

describe("account list query state", () => {
  it("uses the documented defaults", () => {
    expect(parseAccountListQuery("")).toEqual(DEFAULT_ACCOUNT_LIST_QUERY);
    expect(serializeAccountListQuery(DEFAULT_ACCOUNT_LIST_QUERY)).toBe("");
  });

  it("accepts supported status, page, page size, and deleted filters", () => {
    const query = parseAccountListQuery(
      "?page=2&page_size=100&status=inactive&include_deleted=true",
    );

    expect(query).toEqual({
      page: 2,
      pageSize: 100,
      status: "inactive",
      includeDeleted: true,
    });

    expect(serializeAccountListQuery(query)).toBe(
      "?page=2&page_size=100&status=inactive&include_deleted=true",
    );
  });

  it("drops unsupported or invalid URL state", () => {
    expect(
      parseAccountListQuery(
        "?page=0&page_size=101&status=deleted&sort=email&q=secret",
      ),
    ).toEqual(DEFAULT_ACCOUNT_LIST_QUERY);

    expect(
      serializeAccountListQuery({
        page: 3,
        pageSize: 100,
        status: "active",
        includeDeleted: false,
      }),
    ).toBe("?page=3&page_size=100&status=active");
  });

  it("keeps arbitrary valid page sizes at or below the server maximum", () => {
    expect(parseAccountListQuery("?page_size=37").pageSize).toBe(37);
    expect(parseAccountListQuery("?page_size=0").pageSize).toBe(20);
  });
});
