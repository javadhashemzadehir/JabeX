import { describe, expect, it } from "vitest";
import { DEFAULT_LOCALE, getDirection, isLocale } from "./config";

describe("i18n config", () => {
  it("defaults to Persian", () => {
    expect(DEFAULT_LOCALE).toBe("fa");
    expect(getDirection("fa")).toBe("rtl");
  });

  it("maps English to LTR", () => {
    expect(getDirection("en")).toBe("ltr");
  });

  it("validates locales", () => {
    expect(isLocale("fa")).toBe(true);
    expect(isLocale("en")).toBe(true);
    expect(isLocale("de")).toBe(false);
  });
});
