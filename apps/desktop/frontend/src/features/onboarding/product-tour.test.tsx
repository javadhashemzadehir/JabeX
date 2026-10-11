import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useLocaleStore } from "@/lib/i18n/locale-store";
import { useProfileStore } from "@/features/profiles/profile-store";
import { ProductTour } from "./product-tour";
import { useOnboardingTourStore } from "./tour-store";

const push = vi.fn();

vi.mock("next/navigation", () => ({
  usePathname: () => "/",
  useRouter: () => ({ push, replace: vi.fn(), back: vi.fn() }),
}));

const profile = {
  id: "11111111-1111-7111-8111-111111111111",
  displayName: "Test",
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
  lastActiveAt: "2026-01-01T00:00:00Z",
};

function TourHarness() {
  return (
    <>
      <div data-tour="home-welcome">Welcome</div>
      <a data-tour="nav-decks" href="/decks">
        Decks
      </a>
      <button type="button" data-tour="create-deck">
        New deck
      </button>
      <button type="button" data-tour="create-card">
        New card
      </button>
      <a data-tour="nav-review" href="/review">
        Review
      </a>
      <a data-tour="nav-settings" href="/settings">
        Settings
      </a>
      <div data-tour="settings-application">Application</div>
      <div data-tour="header-appearance">Appearance</div>
      <ProductTour />
    </>
  );
}

async function startTour() {
  render(<TourHarness />);
  await waitFor(() => {
    expect(screen.getByRole("heading", { name: "Welcome to JabeX" })).toBeInTheDocument();
  });
}

describe("ProductTour", () => {
  beforeEach(() => {
    push.mockClear();
    window.localStorage.clear();
    useLocaleStore.getState().setLocale("en");
    useOnboardingTourStore.setState({
      startToken: 0,
      startedToken: 0,
      autoStartedProfileId: null,
      dismissed: false,
    });
    useProfileStore.setState({
      desktop: true,
      loading: false,
      initialSetupPending: false,
      active: profile,
      profiles: [profile],
    });
  });

  afterEach(() => {
    cleanup();
    window.localStorage.clear();
  });

  it("moves past the decks step and can be skipped", async () => {
    await startTour();

    fireEvent.click(screen.getByTestId("button-primary"));

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Decks" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("button-skip"));

    await waitFor(() => {
      expect(screen.queryByRole("heading", { name: "Decks" })).not.toBeInTheDocument();
    });
    expect(screen.queryByRole("heading", { name: "Welcome to JabeX" })).not.toBeInTheDocument();
    expect(
      window.localStorage.getItem(
        "jabex.onboarding.tour.completed:11111111-1111-7111-8111-111111111111",
      ),
    ).toBe("true");
  });

  it("advances from the decks step to create-deck without restarting", async () => {
    await startTour();

    fireEvent.click(screen.getByTestId("button-primary"));
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Decks" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("button-primary"));

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Create a deck" })).toBeInTheDocument();
    });
    expect(screen.queryByRole("heading", { name: "Welcome to JabeX" })).not.toBeInTheDocument();
    expect(push).toHaveBeenCalledWith("/decks");
  });
});
