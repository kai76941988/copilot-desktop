(() => {
  if (window.__PAKE_COPILOT_OVERLAY_FIX__) return;
  window.__PAKE_COPILOT_OVERLAY_FIX__ = true;

  const isCopilotHost = () => {
    try {
      return window.location.hostname.toLowerCase() === "copilot.microsoft.com";
    } catch (e) {
      return false;
    }
  };

  const hasMainInput = () =>
    !!document.querySelector("textarea, input[type='text']");

  const restoreRootInteractivity = () => {
    const roots = [
      document.documentElement,
      document.body,
      document.querySelector("#__next"),
      document.querySelector("#app"),
      document.querySelector("main"),
    ].filter(Boolean);

    roots.forEach((el) => {
      if (el.hasAttribute("inert")) el.removeAttribute("inert");
      if (el.getAttribute("aria-hidden") === "true")
        el.removeAttribute("aria-hidden");
      if (el.style.pointerEvents === "none") el.style.pointerEvents = "auto";
      if (el.style.opacity && parseFloat(el.style.opacity) < 1)
        el.style.opacity = "";
      if (el.style.filter) el.style.filter = "";
    });
  };

  const isBlockingOverlay = (el) => {
    if (!el || el.hasAttribute("data-pake-overlay-cleared")) return false;
    const style = getComputedStyle(el);
    if (style.display === "none" || style.visibility === "hidden") return false;
    if (style.pointerEvents === "none") return false;
    if (style.position !== "fixed" && style.position !== "absolute")
      return false;

    const rect = el.getBoundingClientRect();
    if (
      rect.width < window.innerWidth * 0.9 ||
      rect.height < window.innerHeight * 0.9
    ) {
      return false;
    }

    const zIndex = parseInt(style.zIndex || "0", 10);
    if (!Number.isFinite(zIndex) || zIndex < 100) return false;

    const bg = style.backgroundColor;
    const hasBg =
      bg && bg !== "rgba(0, 0, 0, 0)" && bg !== "transparent";
    const hasBackdrop =
      style.backdropFilter && style.backdropFilter !== "none";

    // If the overlay contains visible interactive UI, do not remove it.
    const hasInteractiveChild = !!el.querySelector(
      "button, input, textarea, select, a, [role='dialog'], [aria-modal='true']",
    );

    if (!(hasBg || hasBackdrop) || hasInteractiveChild) return false;

    // Only clear overlays that have been stuck for a short while
    const now = Date.now();
    const firstSeen = parseInt(
      el.getAttribute("data-pake-overlay-first-seen") || "0",
      10,
    );
    if (!firstSeen) {
      el.setAttribute("data-pake-overlay-first-seen", String(now));
      return false;
    }

    return now - firstSeen > 2500;
  };

  const clearBlockingOverlays = () => {
    if (!document.body) return;
    const candidates = Array.from(
      document.body.querySelectorAll("div, section, main, aside"),
    );
    const overlays = candidates.filter(isBlockingOverlay);
    if (!overlays.length) return;

    overlays.forEach((el) => {
      el.setAttribute("data-pake-overlay-cleared", "1");
      el.style.pointerEvents = "none";
      el.style.background = "transparent";
      el.style.opacity = "0";
    });
    restoreRootInteractivity();
  };

  const tryFixOverlay = () => {
    if (!isCopilotHost()) return;
    if (!hasMainInput()) return;
    clearBlockingOverlays();
  };

  document.addEventListener("DOMContentLoaded", () => {
    if (!isCopilotHost()) return;

    setTimeout(tryFixOverlay, 1500);

    const observer = new MutationObserver(() => {
      tryFixOverlay();
    });
    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["class", "style", "aria-hidden", "inert"],
    });

    window.addEventListener("hashchange", () => {
      setTimeout(tryFixOverlay, 500);
    });
  });
})();
