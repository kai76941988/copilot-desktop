(() => {
  try {
    const HOST_OK = /(^|\.)copilot\.microsoft\.com$/i.test(
      window.location.hostname,
    );
    if (!HOST_OK) return;

    const DEBUG =
      window.__PAKE_DEBUG_AUTH__ !== undefined
        ? window.__PAKE_DEBUG_AUTH__
        : true;
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

    const LOGGED_IN_SELECTORS = [
      '[data-testid*="profile"]',
      '[data-testid*="account"]',
      '[data-testid*="user"]',
      '[data-testid*="settings"]',
      '[aria-label*="Account"]',
      '[aria-label*="Profile"]',
      '[aria-label*="Settings"]',
      '[aria-label*="Sign out"]',
      '[aria-label*="Log out"]',
      '[aria-label*="New chat"]',
      '[aria-label*="New conversation"]',
      '[aria-label*="New thread"]',
      'img[alt*="avatar"]',
      'img[alt*="profile"]',
      '[class*="avatar"] img',
      '[class*="profile"] img',
    ];

    const LOGGED_OUT_SELECTORS = [
      'a[href*="login"]',
      'a[href*="signin"]',
      'a[href*="sign-in"]',
      'a[href*="signup"]',
      'button[aria-label*="Sign in"]',
      'button[aria-label*="Log in"]',
      'button[aria-label*="Sign up"]',
    ];

    const AUTH_FORM_SELECTORS = [
      'input[type="email"]',
      'input[type="password"]',
      'input[name="loginfmt"]',
      "#i0116",
      "#i0118",
      'form[action*="login"]',
      '[data-testid*="login"]',
    ];

    const AUTH_UI_CANDIDATE_SELECTORS = [
      ...AUTH_FORM_SELECTORS,
      '[role="dialog"]',
      '[role="menu"]',
      '[aria-modal="true"]',
      '[class*="flyout"]',
      '[class*="popover"]',
      '[class*="menu"]',
      '[class*="account"]',
      '[class*="profile"]',
      '[class*="signin"]',
      '[class*="login"]',
      '[data-testid*="account"]',
      '[data-testid*="profile"]',
      '[data-testid*="signin"]',
      '[data-testid*="login"]',
    ];

    const LOGGED_IN_TEXT_PATTERNS = [
      /new chat/i,
      /new conversation/i,
      /new thread/i,
      /profile/i,
      /account/i,
      /settings/i,
      /sign out/i,
      /log out/i,
      /logout/i,
      /personalize/i,
      /my account/i,
      /我的/i,
      /账户/i,
      /设置/i,
      /退出/i,
      /新建/i,
      /新聊天/i,
    ];

    const LOGGED_OUT_TEXT_PATTERNS = [
      /sign in/i,
      /log in/i,
      /login/i,
      /sign up/i,
      /get started/i,
      /use copilot/i,
      /登录/i,
      /注册/i,
    ];

    const AUTH_PRESERVE_TEXT_PATTERNS = [
      ...LOGGED_IN_TEXT_PATTERNS,
      ...LOGGED_OUT_TEXT_PATTERNS,
      /use another account/i,
      /add account/i,
      /switch account/i,
      /account/i,
      /profile/i,
      /settings/i,
      /sign out/i,
      /log out/i,
      /logout/i,
      /microsoft/i,
      /账户/i,
      /账号/i,
      /切换/i,
      /添加/i,
      /个人/i,
      /退出/i,
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

    let mainReadySince = 0;

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

    function findVisibleBySelectors(selectors) {
      for (const sel of selectors) {
        const list = Array.from(document.querySelectorAll(sel));
        for (const el of list) {
          if (isVisible(el)) return el;
        }
      }
      return null;
    }

    function findVisibleTextMatch(patterns) {
      const nodes = Array.from(document.querySelectorAll("button, a, span, div"));
      for (const el of nodes) {
        if (!isVisible(el)) continue;
        const text = (el.innerText || "").trim();
        if (!text) continue;
        if (patterns.some((p) => p.test(text))) return el;
      }
      return null;
    }

    function elementMatchesSelectors(el, selectors) {
      if (!el || !(el instanceof Element)) return false;
      return selectors.some((sel) => {
        try {
          return el.matches(sel);
        } catch (_) {
          return false;
        }
      });
    }

    function elementContainsSelector(el, selectors) {
      if (!el || !(el instanceof Element)) return false;
      for (const sel of selectors) {
        try {
          if (el.querySelector(sel)) return true;
        } catch (_) {}
      }
      return false;
    }

    function hasInteractiveChildren(el) {
      if (!el || !(el instanceof Element)) return false;
      const interactive = el.querySelectorAll(
        "input, textarea, select, button, a, [role='button'], [tabindex]",
      );
      for (const child of interactive) {
        if (isVisible(child)) return true;
      }
      return false;
    }

    function isAuthUiElement(el) {
      if (!el || !(el instanceof Element)) return false;
      if (!isVisible(el)) return false;

      if (elementMatchesSelectors(el, AUTH_FORM_SELECTORS)) return true;
      if (elementContainsSelector(el, AUTH_FORM_SELECTORS)) return true;

      const label = (el.getAttribute("aria-label") || "").toLowerCase();
      const idClass = `${el.id || ""} ${el.className || ""}`.toLowerCase();
      if (
        /(account|profile|signin|sign-in|login|auth|msal|microsoft|user)/.test(
          `${label} ${idClass}`,
        )
      ) {
        return true;
      }

      const text = (el.innerText || "").trim();
      if (text && AUTH_PRESERVE_TEXT_PATTERNS.some((p) => p.test(text))) {
        return true;
      }

      if (
        elementMatchesSelectors(el, [
          "[role='dialog']",
          "[role='menu']",
          "[aria-modal='true']",
        ]) &&
        hasInteractiveChildren(el)
      ) {
        return true;
      }

      return false;
    }

    function isLoginAccountFlyout(el) {
      if (!el || !(el instanceof Element)) return false;
      if (!isVisible(el)) return false;
      const rect = el.getBoundingClientRect();
      const nearLeft = rect.left < window.innerWidth * 0.35;
      const nearBottom = rect.bottom > window.innerHeight * 0.5;
      const smallPanel =
        rect.width < window.innerWidth * 0.6 &&
        rect.height < window.innerHeight * 0.8;
      return nearLeft && nearBottom && smallPanel && isAuthUiElement(el);
    }

    function shouldPreserveElement(el) {
      if (!el || !(el instanceof Element)) return false;
      if (!isVisible(el)) return false;
      if (isAuthUiElement(el)) return true;
      if (isLoginAccountFlyout(el)) return true;
      return false;
    }

    function getAuthUiElement() {
      for (const sel of AUTH_UI_CANDIDATE_SELECTORS) {
        const list = Array.from(document.querySelectorAll(sel));
        for (const el of list) {
          if (isAuthUiElement(el)) return el;
        }
      }
      return null;
    }

    function ensureAuthFlyoutVisible(reason) {
      const authEl = getAuthUiElement();
      if (!authEl) return 0;

      let fixed = 0;
      try {
        authEl.style.setProperty("z-index", "999999", "important");
        authEl.style.setProperty("pointer-events", "auto", "important");
      } catch (_) {}

      let node = authEl.parentElement;
      while (node && node !== document.documentElement) {
        const style = window.getComputedStyle(node);
        if (style.overflow !== "visible") {
          node.style.setProperty("overflow", "visible", "important");
          fixed++;
        }
        if (style.overflowX !== "visible") {
          node.style.setProperty("overflow-x", "visible", "important");
          fixed++;
        }
        if (style.overflowY !== "visible") {
          node.style.setProperty("overflow-y", "visible", "important");
          fixed++;
        }
        if (style.clipPath && style.clipPath !== "none") {
          node.style.setProperty("clip-path", "none", "important");
          fixed++;
        }
        if (style.contain && style.contain !== "none") {
          node.style.setProperty("contain", "none", "important");
          fixed++;
        }
        if (style.transform && style.transform !== "none") {
          node.style.setProperty("transform", "none", "important");
          fixed++;
        }
        node = node.parentElement;
      }

      try {
        const bodyStyle = window.getComputedStyle(document.body);
        if (bodyStyle.transform && bodyStyle.transform !== "none") {
          document.body.style.setProperty("transform", "none", "important");
          document.body.style.setProperty("transform-origin", "top left", "important");
          document.body.style.setProperty("width", "auto", "important");
          document.body.style.setProperty("height", "auto", "important");
          fixed++;
        }
      } catch (_) {}

      try {
        const htmlZoom = window.localStorage.getItem("htmlZoom");
        if (htmlZoom) {
          document.documentElement.style.setProperty("zoom", htmlZoom, "important");
        }
      } catch (_) {}

      if (fixed > 0) {
        log("auth flyout fix applied", { reason, fixed });
      }
      return fixed;
    }

    function getMainReadyElement() {
      return findVisibleBySelectors(MAIN_READY_SELECTORS);
    }

    function isMainUiVisible() {
      return !!getMainReadyElement();
    }

    function getInputElement() {
      return (
        findVisibleBySelectors([
          "textarea",
          'div[contenteditable="true"]',
          'div[role="textbox"]',
          '[data-testid*="composer"] textarea',
          '[data-testid*="prompt"] textarea',
        ]) || null
      );
    }

    function hasAuthCache() {
      try {
        const lsKeys = Object.keys(window.localStorage || {});
        const ssKeys = Object.keys(window.sessionStorage || {});
        const hasMsalKey = (k) =>
          k.startsWith("msal.") ||
          k.includes("login.microsoftonline") ||
          k.includes("microsoft") ||
          k.includes("aad");
        return (
          lsKeys.some(hasMsalKey) ||
          ssKeys.some(hasMsalKey)
        );
      } catch (_) {
        return false;
      }
    }

    function isLoggedInUiReady() {
      const mainEl = getMainReadyElement();
      if (!mainEl) {
        mainReadySince = 0;
        return false;
      }

      const loggedOutEl =
        findVisibleBySelectors(LOGGED_OUT_SELECTORS) ||
        findVisibleTextMatch(LOGGED_OUT_TEXT_PATTERNS);
      if (loggedOutEl) {
        mainReadySince = 0;
        return false;
      }

      const authFormEl = findVisibleBySelectors(AUTH_FORM_SELECTORS);
      if (authFormEl) {
        mainReadySince = 0;
        return false;
      }

      const loggedInEl =
        findVisibleBySelectors(LOGGED_IN_SELECTORS) ||
        findVisibleTextMatch(LOGGED_IN_TEXT_PATTERNS);
      if (loggedInEl) return true;

      // Fallback: main UI + enabled input for a short period
      const inputEl = getInputElement();
      const inputEnabled =
        inputEl &&
        !inputEl.disabled &&
        inputEl.getAttribute("aria-disabled") !== "true" &&
        !inputEl.readOnly;

      const authCacheReady = hasAuthCache();
      if (inputEnabled && authCacheReady) {
        if (!mainReadySince) mainReadySince = Date.now();
        if (Date.now() - mainReadySince > 10000) return true;
      } else {
        mainReadySince = 0;
      }

      return false;
    }

    function shouldCleanupOverlay() {
      const mainReady = isMainUiVisible();
      const loggedIn = isLoggedInUiReady();
      return mainReady && loggedIn;
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
      if (shouldPreserveElement(el)) return false;
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
      if (shouldPreserveElement(el)) return false;
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
          if (shouldPreserveElement(el)) continue;
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
        if (shouldPreserveElement(el)) {
          continue;
        }
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
      ensureAuthFlyoutVisible(reason);
      const mainReady = isMainUiVisible();
      const loggedInReady = isLoggedInUiReady();
      const allowCleanup = shouldCleanupOverlay();

      log("status", {
        reason,
        url: window.location.href,
        mainReady,
        loggedInReady,
        allowCleanup,
      });

      if (!mainReady) {
        log("skip cleanup, main UI not ready:", reason);
        return 0;
      }
      if (!loggedInReady) {
        log("skip cleanup, logged-in UI not ready:", reason);
        return 0;
      }
      if (!allowCleanup) {
        log("skip cleanup, gate not satisfied:", reason);
        return 0;
      }

      const mainEl = getMainReadyElement();
      if (!mainEl) return 0;

      let changed = 0;
      unlockPageInteraction();
      removeBodyLevelLocks();
      changed += removeKnownOverlays(mainEl);
      changed += sweepBlockingLayers(mainEl);
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

      setTimeout(() => guardedCleanup("t+1200"), 1200);
      setTimeout(() => guardedCleanup("t+2500"), 2500);
      setTimeout(() => guardedCleanup("t+4000"), 4000);
      setTimeout(() => guardedCleanup("t+7000"), 7000);
      setTimeout(() => guardedCleanup("t+12000"), 12000);
    }

    window.pakeOverlayFix = {
      isMainUiVisible,
      isLoggedInUiReady,
      shouldCleanupOverlay,
    };

    init();
  } catch (e) {
    console.error("[CopilotOverlayFix] fatal:", e);
  }
})();
