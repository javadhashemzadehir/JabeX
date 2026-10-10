import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "@/components/providers/app-providers";
import { AppShell } from "./app-shell";

vi.mock("next/navigation", () => ({
  usePathname: () => "/",
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), back: vi.fn() }),
}));

vi.mock("@/features/onboarding/product-tour", () => ({
  ProductTour: () => null,
}));

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

async function waitForShell() {
  await waitFor(() => {
    expect(screen.getByRole("complementary")).toBeInTheDocument();
  });
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

  it("collapses and expands the sidebar", async () => {
    renderShell();
    await waitForShell();
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

  it("places page content in a wide wrapper", async () => {
    renderShell();
    await waitForShell();
    expect(screen.getByText("study content").parentElement).toHaveClass("max-w-7xl");
  });

  it("exposes a skip link to main content", async () => {
    renderShell();
    await waitForShell();
    const skip = screen.getByRole("link", { name: "پرش به محتوای اصلی" });
    expect(skip).toHaveAttribute("href", "#main-content");
    expect(screen.getByRole("main")).toHaveAttribute("id", "main-content");
  });
});
