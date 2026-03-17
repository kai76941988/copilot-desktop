(() => {
  try {
    const HOST_OK = /(^|\.)copilot\.microsoft\.com$/i.test(
      window.location.hostname,
    );
    if (!HOST_OK) return;

    const DEBUG = true;
    const TAG = "[CopilotOverlayFix]";

    const MAIN_READY_SELECTORS = [
      "textarea",
      'div[contenteditable="true"]',
      'div[role="textbox"]',
      '[data-testid*="composer"]',
      '[data-testid*="prompt"]',
      '[class*="composer"]',
      '[class*="prompt"]',
      "main",
      "#root main",
    ];

    const DIRECT_OVERLAY_SELECTORS = [
      '[aria-modal="true"]',
      '[role="dialog"]',
      '[class*="overlay"]',
      '[class*="backdrop"]',
      '[class*="modal"]',
      '[class*="scrim"]',
      '[class*="mask"]',
      '[class*="loading"]',
      '[class*="spinner"]',
      '[class*="splash"]',
      '[id*="overlay"]',
      '[id*="backdrop"]',
      '[id*="modal"]',
      '[id*="loading"]',
      '[id*="spinner"]',
    ];

    function log(...args) {
      if (DEBUG) console.log(TAG, ...args);
    }

    function isVisible(el) {
      if (!el || !(el instanceof Element)) return false;
      const style = window.getComputedStyle(el);
      const rect = el.getBoundingClientRect();
      if (style.display === "none") return false;
      if (style.visibility === "hidden") return false;
      if (parseFloat(style.opacity || "1") <= 0.01) return false;
      if (rect.width <= 0 || rect.height <= 0) return false;
      return true;
    }

    function getMainReadyElement() {
      for (const sel of MAIN_READY_SELECTORS) {
        const list = Array.from(document.querySelectorAll(sel));
        for (const el of list) {
          if (isVisible(el)) return el;
        }
      }
      return null;
    }

    function safeUnlockElement(el) {
      if (!el || !(el instanceof Element)) return;
      try {
        el.style.setProperty("pointer-events", "auto", "important");
        el.style.setProperty("overflow", "auto", "important");
        el.style.setProperty("filter", "none", "important");
        el.style.setProperty("opacity", "1", "important");
        el.style.setProperty("visibility", "visible", "important");
        el.removeAttribute("inert");
        el.removeAttribute("aria-busy");
      } catch (_) {}
    }

    function unlockPageInteraction() {
      safeUnlockElement(document.documentElement);
      safeUnlockElement(document.body);

      const rootCandidates = [
        document.getElementById("root"),
        document.querySelector("#app"),
        document.querySelector("main"),
        document.querySelector("[data-app-root]"),
        document.querySelector('[class*="app"]'),
      ].filter(Boolean);

      rootCandidates.forEach(safeUnlockElement);

      // Some frameworks lock body/html scrolling; restore when main UI is ready.
      try {
        document.documentElement.style.setProperty("overflow", "auto", "important");
        document.body.style.setProperty("overflow", "auto", "important");
        document.body.style.setProperty(
          "pointer-events",
          "auto",
          "important",
        );
      } catch (_) {}
    }

    function hideElement(el, reason) {
      if (!el || !(el instanceof Element)) return false;
      if (el.dataset.__copilotOverlayFixed === "1") return false;

      el.dataset.__copilotOverlayFixed = "1";

      try {
        el.removeAttribute("inert");
      } catch (_) {}
      try {
        el.removeAttribute("aria-busy");
      } catch (_) {}

      try {
        el.style.setProperty("display", "none", "important");
        el.style.setProperty("visibility", "hidden", "important");
        el.style.setProperty("opacity", "0", "important");
        el.style.setProperty("pointer-events", "none", "important");
      } catch (_) {}

      log("hide:", reason, el);
      return true;
    }

    function rectNearCenter(rect) {
      const cx = window.innerWidth / 2;
      const cy = window.innerHeight / 2;
      const rcx = rect.left + rect.width / 2;
      const rcy = rect.top + rect.height / 2;
      return Math.abs(rcx - cx) < 80 && Math.abs(rcy - cy) < 80;
    }

    function isLikelyCenterSpinner(el) {
      if (!isVisible(el)) return false;
      const rect = el.getBoundingClientRect();
      if (rect.width > 120 || rect.height > 120) return false;
      if (!rectNearCenter(rect)) return false;

      const style = window.getComputedStyle(el);
      const cls = `${el.className || ""} ${el.id || ""}`.toLowerCase();

      const keyword =
        cls.includes("spinner") ||
        cls.includes("loading") ||
        cls.includes("loader") ||
        cls.includes("progress") ||
        cls.includes("dot");

      const roundLike =
        parseFloat(style.borderRadius || "0") > 8 ||
        style.borderRadius === "50%";

      return keyword || roundLike;
    }

    function isLikelyBlockingOverlay(el, mainEl) {
      if (!el || !(el instanceof Element)) return false;
      if (!isVisible(el)) return false;
      if (el === mainEl) return false;
      if (mainEl && (el === mainEl || el.contains(mainEl))) return false;

      const style = window.getComputedStyle(el);
      const rect = el.getBoundingClientRect();

      const pos = style.position;
      const z = parseInt(style.zIndex || "0", 10);
      const areaEnough =
        rect.width >= window.innerWidth * 0.35 &&
        rect.height >= window.innerHeight * 0.35;

      const fixedLike =
        pos === "fixed" || pos === "absolute" || pos === "sticky";

      const blocksPointer = style.pointerEvents !== "none";

      const hasOverlayKeyword =
        `${el.className || ""} ${el.id || ""}`.toLowerCase().match(
          /(overlay|backdrop|modal|mask|scrim|loading|spinner|splash|loader)/,
        );

      const darkBg =
        style.backgroundColor &&
        style.backgroundColor !== "rgba(0, 0, 0, 0)" &&
        style.backgroundColor !== "transparent";

      const modalLike =
        el.getAttribute("aria-modal") === "true" ||
        el.getAttribute("role") === "dialog";

      return (
        areaEnough &&
        fixedLike &&
        blocksPointer &&
        (z >= 20 || modalLike || !!hasOverlayKeyword || !!darkBg)
      );
    }

    function removeKnownOverlays(mainEl) {
      let changed = 0;

      for (const sel of DIRECT_OVERLAY_SELECTORS) {
        const nodes = Array.from(document.querySelectorAll(sel));
        for (const el of nodes) {
          if (!isVisible(el)) continue;
          if (mainEl && (el === mainEl || el.contains(mainEl))) continue;
          if (hideElement(el, `direct-selector:${sel}`)) changed++;
        }
      }

      return changed;
    }

    function sweepBlockingLayers(mainEl) {
      let changed = 0;
      const all = Array.from(document.body.querySelectorAll("*"));

      for (const el of all) {
        if (isLikelyBlockingOverlay(el, mainEl)) {
          if (hideElement(el, "blocking-overlay")) changed++;
        } else if (isLikelyCenterSpinner(el)) {
          if (hideElement(el, "center-spinner")) changed++;
        }
      }

      return changed;
    }

    function removeBodyLevelLocks() {
      const attrs = ["inert", "aria-busy"];
      for (const attr of attrs) {
        try {
          document.documentElement.removeAttribute(attr);
        } catch (_) {}
        try {
          document.body.removeAttribute(attr);
        } catch (_) {}
      }

      const lockClasses = [
        "modal-open",
        "overflow-hidden",
        "pointer-events-none",
        "loading",
        "busy",
        "blocked",
      ];

      try {
        lockClasses.forEach((c) => document.documentElement.classList.remove(c));
        lockClasses.forEach((c) => document.body.classList.remove(c));
      } catch (_) {}
    }

    function cleanupOnce(reason) {
      const mainEl = getMainReadyElement();

      // Do nothing if main UI is not ready to avoid breaking login dialogs.
      if (!mainEl) {
        log("skip cleanup, main not ready yet:", reason);
        return 0;
      }

      let changed = 0;

      unlockPageInteraction();
      removeBodyLevelLocks();

      changed += removeKnownOverlays(mainEl);
      changed += sweepBlockingLayers(mainEl);

      // Ensure the page stays interactive after cleanup.
      unlockPageInteraction();
      removeBodyLevelLocks();

      log(`cleanup(${reason}) done, changed =`, changed);
      return changed;
    }

    let highFreqTimer = null;
    let lowFreqTimer = null;
    let observer = null;
    let lastRun = 0;

    function guardedCleanup(reason) {
      const now = Date.now();
      if (now - lastRun < 150) return;
      lastRun = now;

      try {
        cleanupOnce(reason);
      } catch (err) {
        console.error(TAG, "cleanup error:", err);
      }
    }

    function startHighFreqRetry(label) {
      if (highFreqTimer) clearInterval(highFreqTimer);

      let count = 0;
      highFreqTimer = setInterval(() => {
        count++;
        guardedCleanup(`${label}:hf:${count}`);

        // High-frequency checks for the first 15 seconds.
        if (count >= 30) {
          clearInterval(highFreqTimer);
          highFreqTimer = null;
        }
      }, 500);
    }

    function startLowFreqRetry() {
      if (lowFreqTimer) return;
      lowFreqTimer = setInterval(() => {
        guardedCleanup("low-freq");
      }, 3000);
    }

    function hookHistory() {
      const rawPush = history.pushState;
      const rawReplace = history.replaceState;

      history.pushState = function (...args) {
        const ret = rawPush.apply(this, args);
        setTimeout(() => {
          guardedCleanup("pushState");
          startHighFreqRetry("pushState");
        }, 50);
        return ret;
      };

      history.replaceState = function (...args) {
        const ret = rawReplace.apply(this, args);
        setTimeout(() => {
          guardedCleanup("replaceState");
          startHighFreqRetry("replaceState");
        }, 50);
        return ret;
      };
    }

    function startObserver() {
      if (observer) observer.disconnect();

      observer = new MutationObserver(() => {
        guardedCleanup("mutation");
      });

      observer.observe(document.documentElement || document.body, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["class", "style", "aria-busy", "inert"],
      });
    }

    function init() {
      log("init");

      hookHistory();
      startObserver();

      // Immediate attempt.
      guardedCleanup("init");
      startHighFreqRetry("init");
      startLowFreqRetry();

      window.addEventListener("load", () => {
        guardedCleanup("load");
        startHighFreqRetry("load");
      });

      window.addEventListener("pageshow", () => {
        guardedCleanup("pageshow");
        startHighFreqRetry("pageshow");
      });

      window.addEventListener("focus", () => {
        guardedCleanup("focus");
        startHighFreqRetry("focus");
      });

      window.addEventListener("hashchange", () => {
        guardedCleanup("hashchange");
        startHighFreqRetry("hashchange");
      });

      window.addEventListener("popstate", () => {
        guardedCleanup("popstate");
        startHighFreqRetry("popstate");
      });

      document.addEventListener("visibilitychange", () => {
        if (!document.hidden) {
          guardedCleanup("visibilitychange");
          startHighFreqRetry("visibilitychange");
        }
      });

      document.addEventListener("DOMContentLoaded", () => {
        guardedCleanup("DOMContentLoaded");
        startHighFreqRetry("DOMContentLoaded");
      });

      // Extra tolerance for async rendering after login.
      setTimeout(() => guardedCleanup("t+1200"), 1200);
      setTimeout(() => guardedCleanup("t+2500"), 2500);
      setTimeout(() => guardedCleanup("t+4000"), 4000);
      setTimeout(() => guardedCleanup("t+7000"), 7000);
      setTimeout(() => guardedCleanup("t+12000"), 12000);
    }

    init();
  } catch (e) {
    console.error("[CopilotOverlayFix] fatal:", e);
  }
})();
