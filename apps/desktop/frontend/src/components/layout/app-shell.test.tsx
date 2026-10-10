import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppProviders } from "@/components/providers/app-providers";
import { AppShell } from "./app-shell";

const STORAGE_KEY = "jabex.sidebar.collapsed";

function renderShell() {
  return render(
    <AppProviders>
      <AppShell>
        <p>study content</p>
      </AppShell>
    </AppProviders>,
  );
}

function installMatchMedia() {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    configurable: true,
    value: (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: () => undefined,
      removeListener: () => undefined,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
      dispatchEvent: () => false,
    }),
  });
}

describe("AppShell", () => {
  beforeEach(() => {
    installMatchMedia();
    window.localStorage.clear();
  });

  afterEach(() => {
    cleanup();
    window.localStorage.clear();
  });

  it("collapses and expands the sidebar", () => {
    renderShell();
    const sidebar = screen.getByRole("complementary");
    expect(sidebar).toHaveClass("w-60");

    fireEvent.click(screen.getByRole("button", { name: "جمع کردن نوار کناری" }));
    expect(sidebar).toHaveClass("w-16");
    expect(screen.getByRole("link", { name: "خانه" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "باز کردن نوار کناری" }));
    expect(sidebar).toHaveClass("w-60");
    expect(window.localStorage.getItem(STORAGE_KEY)).toBe("false");
  });

  it("restores a collapsed sidebar", async () => {
    window.localStorage.setItem(STORAGE_KEY, "true");
    renderShell();

    await waitFor(() => {
      expect(screen.getByRole("complementary")).toHaveClass("w-16");
    });
  });

  it("places page content in a wide wrapper", () => {
    renderShell();
    expect(screen.getByText("study content").parentElement).toHaveClass("max-w-7xl");
  });
});
