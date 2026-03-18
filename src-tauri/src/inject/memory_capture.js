(function () {
  try {
    const HOST_OK = /(^|\.)copilot\.microsoft\.com$/i.test(location.hostname);
    if (!HOST_OK) return;

    const TAG = "[PakeMemoryCapture]";
    const TAURI = window.__TAURI__;
    if (!TAURI || !TAURI.invoke) {
      console.warn(TAG, "Tauri invoke not available");
      return;
    }

    const PROJECT_KEY = "pake_memory_project_id";
    const SESSION_KEY = "pake_memory_session_id";

    function getOrCreateSessionId() {
      let id = sessionStorage.getItem(SESSION_KEY);
      if (!id) {
        id =
          "sess_" +
          Date.now().toString(36) +
          "_" +
          Math.random().toString(36).slice(2, 8);
        sessionStorage.setItem(SESSION_KEY, id);
      }
      return id;
    }

    function getProjectId() {
      return localStorage.getItem(PROJECT_KEY) || "default";
    }

    function hashText(str) {
      let h = 0;
      for (let i = 0; i < str.length; i++) {
        h = (h << 5) - h + str.charCodeAt(i);
        h |= 0;
      }
      return String(h);
    }

    const seen = new Set();
    const assistantLastText = new WeakMap();
    const assistantTimers = new WeakMap();

    function recordMessage(role, content, meta) {
      const text = (content || "").trim();
      if (!text) return;

      const key = role + ":" + hashText(text);
      if (seen.has(key)) return;
      seen.add(key);

      const payload = {
        role,
        content: text,
        project_id: getProjectId(),
        session_id: getOrCreateSessionId(),
        source: "copilot-webview",
        created_at: Date.now(),
        metadata_json: meta ? JSON.stringify(meta) : null,
      };

      TAURI.invoke("memory_record_message", payload).catch((err) => {
        console.warn(TAG, "record failed:", err);
      });
    }

    function getActiveInputText() {
      const active = document.activeElement;
      if (!active) return "";
      if (active.tagName === "TEXTAREA" || active.tagName === "INPUT") {
        return active.value || "";
      }
      if (active.isContentEditable) {
        return active.innerText || "";
      }
      return "";
    }

    function isSendButton(el) {
      if (!el) return false;
      const txt = (el.innerText || "").toLowerCase();
      const label = (el.getAttribute("aria-label") || "").toLowerCase();
      const cls = (el.className || "").toLowerCase();
      return (
        txt.includes("send") ||
        txt.includes("发送") ||
        label.includes("send") ||
        label.includes("发送") ||
        cls.includes("send") ||
        cls.includes("submit")
      );
    }

    document.addEventListener(
      "keydown",
      (e) => {
        if (e.key !== "Enter" || e.shiftKey || e.isComposing) return;
        const text = getActiveInputText();
        if (text) {
          recordMessage("user", text, { source: "input-keydown" });
        }
      },
      true
    );

    document.addEventListener(
      "click",
      (e) => {
        const target = e.target;
        if (!target) return;
        if (isSendButton(target) || isSendButton(target.closest && target.closest("button"))) {
          const text = getActiveInputText();
          if (text) {
            recordMessage("user", text, { source: "input-click" });
          }
        }
      },
      true
    );

    function findNewChatButton() {
      const candidates = Array.from(document.querySelectorAll("button, a"));
      for (const el of candidates) {
        const text = (el.innerText || "").toLowerCase();
        const label = (el.getAttribute("aria-label") || "").toLowerCase();
        const data = (el.getAttribute("data-testid") || "").toLowerCase();
        if (
          text.includes("new chat") ||
          text.includes("new conversation") ||
          text.includes("新建") ||
          label.includes("new chat") ||
          label.includes("新建") ||
          data.includes("new-chat") ||
          data.includes("new_conversation")
        ) {
          return el;
        }
      }
      return null;
    }

    function findInputBox() {
      const selectors = [
        "textarea",
        'div[contenteditable="true"]',
        'div[role="textbox"]',
        '[data-testid*="composer"]',
        '[data-testid*="prompt"]',
      ];
      for (const sel of selectors) {
        const nodes = Array.from(document.querySelectorAll(sel));
        for (const el of nodes) {
          if (isVisible(el)) return el;
        }
      }
      return null;
    }

    function setInputText(el, text) {
      if (!el) return false;
      try {
        el.focus();
        if (el.tagName === "TEXTAREA" || el.tagName === "INPUT") {
          el.value = text;
          el.dispatchEvent(new Event("input", { bubbles: true }));
          el.dispatchEvent(new Event("change", { bubbles: true }));
          return true;
        }
        if (el.isContentEditable) {
          el.innerText = text;
          el.dispatchEvent(
            new InputEvent("input", { bubbles: true, data: text, inputType: "insertText" })
          );
          return true;
        }
      } catch (e) {
        console.warn(TAG, "setInputText failed", e);
      }
      return false;
    }

    function handleContextPack(payload) {
      const text = (payload && payload.text) || payload || "";
      if (!text) return;
      const forceNewChat = !!(payload && payload.forceNewChat);
      if (forceNewChat) {
        const btn = findNewChatButton();
        if (btn) {
          btn.click();
        }
        setTimeout(() => {
          const input = findInputBox();
          if (input) {
            setInputText(input, text);
          }
        }, 800);
        return;
      }

      const input = findInputBox();
      if (input) {
        setInputText(input, text);
      }
    }

    if (TAURI.event && TAURI.event.listen) {
      TAURI.event.listen("memory_set_project", (event) => {
        try {
          const projectId = event?.payload?.project_id || event?.payload;
          if (projectId) {
            localStorage.setItem(PROJECT_KEY, projectId);
            sessionStorage.removeItem(SESSION_KEY);
            console.log(TAG, "active project set:", projectId);
          }
        } catch (err) {
          console.warn(TAG, "set project failed:", err);
        }
      });

      TAURI.event.listen("memory_context_pack", (event) => {
        handleContextPack(event?.payload);
      });
    }

    function isVisible(el) {
      if (!el || !(el instanceof Element)) return false;
      const style = window.getComputedStyle(el);
      if (style.display === "none" || style.visibility === "hidden") return false;
      const rect = el.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return false;
      return true;
    }

    function inferRole(el) {
      const attrs =
        (el.getAttribute("data-testid") || "") +
        " " +
        (el.getAttribute("data-role") || "") +
        " " +
        (el.getAttribute("data-author") || "") +
        " " +
        (el.getAttribute("aria-label") || "") +
        " " +
        (el.className || "");
      const lower = attrs.toLowerCase();

      if (lower.includes("user") || lower.includes("me") || lower.includes("you")) {
        return "user";
      }
      if (
        lower.includes("assistant") ||
        lower.includes("copilot") ||
        lower.includes("bot")
      ) {
        return "assistant";
      }

      if (el.closest && el.closest('[data-testid*="user"]')) return "user";
      if (el.closest && el.closest('[data-testid*="assistant"]')) return "assistant";
      if (el.closest && el.closest('[class*="assistant"]')) return "assistant";
      return "";
    }

    const MESSAGE_SELECTORS = [
      '[data-testid*="message"]',
      '[data-testid*="chat-message"]',
      '[data-testid*="assistant"]',
      '[data-testid*="bot"]',
      'div[role="article"]',
      'article',
      'div[aria-live="polite"]',
      'div[aria-live="assertive"]',
    ];

    let scanTimer = null;
    function scheduleScan() {
      if (scanTimer) clearTimeout(scanTimer);
      scanTimer = setTimeout(scanMessages, 450);
    }

    function scanMessages() {
      const roots = [
        document.querySelector("main"),
        document.querySelector('[role="main"]'),
        document.body,
      ].filter(Boolean);

      const candidates = new Set();
      for (const root of roots) {
        for (const sel of MESSAGE_SELECTORS) {
          root.querySelectorAll(sel).forEach((el) => candidates.add(el));
        }
      }

      candidates.forEach((el) => {
        if (!isVisible(el)) return;
        const text = (el.innerText || "").trim();
        if (!text || text.length < 2) return;

        const role = inferRole(el);
        if (!role) return;

        if (role === "assistant") {
          const last = assistantLastText.get(el) || "";
          if (text === last) return;

          assistantLastText.set(el, text);
          const existingTimer = assistantTimers.get(el);
          if (existingTimer) clearTimeout(existingTimer);

          const timer = setTimeout(() => {
            const current = (el.innerText || "").trim();
            if (current && current === text) {
              recordMessage("assistant", current, { source: "dom-stable" });
            }
          }, 1200);
          assistantTimers.set(el, timer);
        } else if (role === "user") {
          recordMessage("user", text, { source: "dom" });
        }
      });
    }

    const observer = new MutationObserver(() => scheduleScan());
    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      characterData: true,
    });

    scheduleScan();
    console.log(TAG, "initialized");
  } catch (e) {
    console.error("[PakeMemoryCapture] fatal:", e);
  }
})();
