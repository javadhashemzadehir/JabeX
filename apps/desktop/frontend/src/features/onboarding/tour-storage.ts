const COMPLETED_PREFIX = "jabex.onboarding.tour.completed:";

function keyForProfile(profileId: string) {
  return `${COMPLETED_PREFIX}${profileId}`;
}

export function isProductTourCompleted(profileId: string): boolean {
  try {
    return window.localStorage.getItem(keyForProfile(profileId)) === "true";
  } catch {
    return false;
  }
}

export function markProductTourCompleted(profileId: string) {
  try {
    window.localStorage.setItem(keyForProfile(profileId), "true");
  } catch {
    // Ignore storage failures (private mode, etc.)
  }
}

export function clearProductTourCompleted(profileId: string) {
  try {
    window.localStorage.removeItem(keyForProfile(profileId));
  } catch {
    // Ignore storage failures
  }
}
