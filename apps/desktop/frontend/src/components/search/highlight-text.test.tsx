import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { HighlightText } from "./highlight-text";

describe("HighlightText", () => {
  it("highlights english matches", () => {
    render(<HighlightText text="Learn Java programming" query="java" />);
    expect(screen.getByText("Java").tagName).toBe("MARK");
  });

  it("highlights persian matches", () => {
    render(<HighlightText text="درس فلسفه غرب" query="فلسفه" />);
    expect(screen.getByText("فلسفه").tagName).toBe("MARK");
  });
});
