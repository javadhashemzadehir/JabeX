import { create } from "zustand";

interface OnboardingTourState {
  /** Increments when the user explicitly starts or restarts the tour. */
  startToken: number;
  /** Highest startToken already passed to Joyride. Survives route remounts. */
  startedToken: number;
  /** Profile that already received the automatic first-run start this session. */
  autoStartedProfileId: string | null;
  /** User finished or skipped; blocks another automatic start until they replay. */
  dismissed: boolean;
  requestStart: () => void;
  markStarted: (token: number) => void;
  markAutoStarted: (profileId: string) => void;
  dismiss: () => void;
}

export const useOnboardingTourStore = create<OnboardingTourState>((set) => ({
  startToken: 0,
  startedToken: 0,
  autoStartedProfileId: null,
  dismissed: false,
  requestStart: () =>
    set((state) => ({
      startToken: state.startToken + 1,
      dismissed: false,
    })),
  markStarted: (token) => set({ startedToken: token }),
  markAutoStarted: (profileId) => set({ autoStartedProfileId: profileId }),
  dismiss: () => set({ dismissed: true }),
}));
