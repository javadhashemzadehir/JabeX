import { create } from "zustand";

interface OnboardingTourState {
  /** Increments when the user explicitly starts or restarts the tour. */
  startToken: number;
  requestStart: () => void;
}

export const useOnboardingTourStore = create<OnboardingTourState>((set) => ({
  startToken: 0,
  requestStart: () => set((state) => ({ startToken: state.startToken + 1 })),
}));
