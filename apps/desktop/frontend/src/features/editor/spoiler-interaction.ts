/** Toggle Telegram-style spoiler reveal on click / keyboard. */
export function bindSpoilerInteractions(root: HTMLElement): () => void {
  function toggleTarget(target: HTMLElement) {
    const revealed = target.hasAttribute("data-revealed");
    if (revealed) {
      target.removeAttribute("data-revealed");
      target.setAttribute("aria-expanded", "false");
    } else {
      target.setAttribute("data-revealed", "");
      target.setAttribute("aria-expanded", "true");
    }
  }

  function onClick(event: MouseEvent) {
    const el = (event.target as HTMLElement | null)?.closest?.(
      "[data-spoiler], [data-spoiler-block]",
    ) as HTMLElement | null;
    if (!el || !root.contains(el)) return;
    // In editable mode, allow selecting text; only toggle when not selecting.
    const selection = window.getSelection();
    if (selection && !selection.isCollapsed) return;
    event.preventDefault();
    toggleTarget(el);
  }

  function onKeyDown(event: KeyboardEvent) {
    if (event.key !== "Enter" && event.key !== " ") return;
    const el = (event.target as HTMLElement | null)?.closest?.(
      "[data-spoiler], [data-spoiler-block]",
    ) as HTMLElement | null;
    if (!el || !root.contains(el)) return;
    event.preventDefault();
    toggleTarget(el);
  }

  root.addEventListener("click", onClick);
  root.addEventListener("keydown", onKeyDown);
  return () => {
    root.removeEventListener("click", onClick);
    root.removeEventListener("keydown", onKeyDown);
  };
}
