/** Wait until a CSS selector matches (for tour navigation between routes). */
export function waitForSelector(
  selector: string,
  timeoutMs = 4000,
): Promise<Element> {
  const existing = document.querySelector(selector);
  if (existing) return Promise.resolve(existing);

  return new Promise((resolve, reject) => {
    let settled = false;
    const finish = (result: Element | Error) => {
      if (settled) return;
      settled = true;
      observer.disconnect();
      window.clearTimeout(timer);
      if (result instanceof Error) reject(result);
      else resolve(result);
    };

    const observer = new MutationObserver(() => {
      const el = document.querySelector(selector);
      if (el) finish(el);
    });
    observer.observe(document.body, { childList: true, subtree: true });

    const timer = window.setTimeout(() => {
      const el = document.querySelector(selector);
      if (el) finish(el);
      else finish(new Error(`Timed out waiting for ${selector}`));
    }, timeoutMs);
  });
}
