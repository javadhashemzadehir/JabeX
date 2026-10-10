import { describe, expect, it } from "vitest";

import { formatBytes } from "./format-bytes";

describe("formatBytes", () => {
  it("formats small and large sizes", () => {
    expect(formatBytes(0, "en")).toBe("0 B");
    expect(formatBytes(512, "en")).toBe("512 B");
    expect(formatBytes(2048, "en")).toBe("2 KB");
    expect(formatBytes(5 * 1024 * 1024, "en")).toBe("5 MB");
  });
});
