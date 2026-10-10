"use client";

import { create } from "zustand";

/**
 * UI-only signal for opening the header profile switcher (e.g. from tray).
 * Does not hold profile data — that stays in `profile-store` / Rust services.
 */
interface ProfileSwitcherUiState {
  menuOpen: boolean;
  openMenu: () => void;
  setMenuOpen: (open: boolean) => void;
}

export const useProfileSwitcherUi = create<ProfileSwitcherUiState>((set) => ({
  menuOpen: false,
  openMenu: () => set({ menuOpen: true }),
  setMenuOpen: (menuOpen) => set({ menuOpen }),
}));
