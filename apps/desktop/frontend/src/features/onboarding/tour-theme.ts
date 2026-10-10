/** Resolve design-token colors for Joyride tooltips (light/dark aware). */
export function readTourTheme() {
  if (typeof window === "undefined") {
    return {
      primaryColor: "#2f6b5a",
      backgroundColor: "#ffffff",
      textColor: "#222222",
      overlayColor: "rgba(0, 0, 0, 0.45)",
      arrowColor: "#ffffff",
    };
  }
  const styles = getComputedStyle(document.documentElement);
  const read = (name: string, fallback: string) => {
    const value = styles.getPropertyValue(name).trim();
    return value || fallback;
  };
  const background = read("--popover", read("--background", "#ffffff"));
  return {
    primaryColor: read("--primary", "#2f6b5a"),
    backgroundColor: background,
    textColor: read("--popover-foreground", read("--foreground", "#222222")),
    overlayColor: "color-mix(in oklab, var(--foreground) 45%, transparent)",
    arrowColor: background,
  };
}
