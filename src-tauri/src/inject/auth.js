(() => {
  const DEBUG =
    window.__PAKE_DEBUG_AUTH__ !== undefined
      ? window.__PAKE_DEBUG_AUTH__
      : true;
  const TAG = "[PakeAuth]";

  function log(...args) {
    if (DEBUG) console.log(TAG, ...args);
  }

  function toUrl(url, baseUrl = window.location.href) {
    try {
      return new URL(url, baseUrl);
    } catch (_) {
      return null;
    }
  }

  function hostMatches(hostname, pattern) {
    try {
      return pattern.test(hostname);
    } catch (_) {
      return false;
    }
  }

  function isCopilotUrl(url) {
    const u = toUrl(url);
    if (!u) return false;
    return /(^|\.)copilot\.microsoft\.com$/i.test(u.hostname);
  }

  const MICROSOFT_AUTH_HOST_PATTERNS = [
    /(^|\.)login\.microsoftonline\.com$/i,
    /(^|\.)login\.microsoftonline\.(com|cn|us|de)$/i,
    /(^|\.)login\.microsoft\.com$/i,
    /(^|\.)login\.windows\.net$/i,
    /(^|\.)login\.live\.com$/i,
    /(^|\.)account\.live\.com$/i,
    /(^|\.)signup\.live\.com$/i,
    /(^|\.)account\.microsoft\.com$/i,
    /(^|\.)microsoftonline\.(com|cn|us|de)$/i,
    /(^|\.)microsoftonline-p\.com$/i,
    /(^|\.)msauth\.net$/i,
    /(^|\.)msftauth\.net$/i,
    /(^|\.)msauthimages\.net$/i,
    /(^|\.)msftauthimages\.net$/i,
    /(^|\.)aadcdn\.msauth\.net$/i,
    /(^|\.)aadcdn\.msftauth\.net$/i,
  ];

  const INTERNAL_ALLOWED_HOST_PATTERNS = [
    /(^|\.)copilot\.microsoft\.com$/i,
    /(^|\.)microsoft\.com$/i,
    /(^|\.)office\.com$/i,
    /(^|\.)live\.com$/i,
    /(^|\.)microsoftonline\.(com|cn|us|de)$/i,
    /(^|\.)microsoftonline-p\.com$/i,
    /(^|\.)msauth\.net$/i,
    /(^|\.)msftauth\.net$/i,
    /(^|\.)windows\.net$/i,
  ];

  const AUTH_PATH_PATTERNS = [
    /\/oauth/i,
    /\/oauth2/i,
    /\/authorize/i,
    /\/token/i,
    /\/callback/i,
    /\/redirect/i,
    /\/kmsi/i,
    /\/consent/i,
    /\/login/i,
    /\/signin/i,
    /\/logout/i,
    /\/mfa/i,
    /\/saml/i,
    /\/wsfed/i,
    /\/federation/i,
  ];

  function isMicrosoftAuthUrl(url) {
    const u = toUrl(url);
    if (!u) return false;
    const host = u.hostname.toLowerCase();
    const path = u.pathname.toLowerCase();
    const full = u.href.toLowerCase();

    const hostIsAuth = MICROSOFT_AUTH_HOST_PATTERNS.some((p) =>
      hostMatches(host, p),
    );
    if (hostIsAuth) return true;

    const isMicrosoftRelatedHost =
      /(^|\.)microsoft\.com$/i.test(host) ||
      /(^|\.)office\.com$/i.test(host) ||
      /(^|\.)live\.com$/i.test(host) ||
      /(^|\.)microsoftonline\./i.test(host);

    const hasAuthPath = AUTH_PATH_PATTERNS.some(
      (p) => p.test(path) || p.test(full),
    );

    return isMicrosoftRelatedHost && hasAuthPath;
  }

  function isInternalAllowedUrl(url) {
    const u = toUrl(url);
    if (!u) return false;
    const host = u.hostname.toLowerCase();
    if (isCopilotUrl(u.href)) return true;
    if (isMicrosoftAuthUrl(u.href)) return true;
    return INTERNAL_ALLOWED_HOST_PATTERNS.some((p) => hostMatches(host, p));
  }

  function isAuthPopupName(name) {
    const normalized = (name || "").toLowerCase();
    if (!normalized) return false;
    const keywords = [
      "msal",
      "msalv2",
      "microsoft",
      "aad",
      "msauth",
      "login",
      "signin",
      "oauth",
      "oauth2",
      "auth",
      "wlid",
      "live",
    ];
    return keywords.some((k) => normalized.includes(k));
  }

  function shouldForceSameWindowAuth(url) {
    return isMicrosoftAuthUrl(url);
  }

  function matchesAuthUrl(url, baseUrl = window.location.href) {
    const u = toUrl(url, baseUrl);
    if (!u) return false;
    if (isMicrosoftAuthUrl(u.href)) return true;

    const hostname = u.hostname.toLowerCase();
    const pathname = u.pathname.toLowerCase();
    const fullUrl = u.href.toLowerCase();

    const genericPatterns = [
      /accounts\.google\.com/,
      /accounts\.google\.[a-z]+/,
      /github\.com\/login/,
      /facebook\.com\/.*\/dialog/,
      /twitter\.com\/oauth/,
      /appleid\.apple\.com/,
      /\/oauth\//,
      /\/auth\//,
      /\/authorize/,
      /\/login\/oauth/,
      /\/signin/,
      /\/login/,
      /servicelogin/,
      /\/o\/oauth2/,
    ];

    return genericPatterns.some(
      (pattern) =>
        pattern.test(hostname) ||
        pattern.test(pathname) ||
        pattern.test(fullUrl),
    );
  }

  function isAuthLink(url) {
    return matchesAuthUrl(url);
  }

  function isAuthPopup(url, name) {
    if (isAuthPopupName(name)) return true;
    return matchesAuthUrl(url);
  }

  // Expose helpers for other inject scripts
  window.pakeAuth = {
    isCopilotUrl,
    isMicrosoftAuthUrl,
    isInternalAllowedUrl,
    shouldForceSameWindowAuth,
    isAuthPopupName,
  };

  window.isCopilotUrl = isCopilotUrl;
  window.isMicrosoftAuthUrl = isMicrosoftAuthUrl;
  window.isInternalAllowedUrl = isInternalAllowedUrl;
  window.shouldForceSameWindowAuth = shouldForceSameWindowAuth;
  window.matchesAuthUrl = matchesAuthUrl;
  window.isAuthLink = isAuthLink;
  window.isAuthPopup = isAuthPopup;
  window.__PAKE_DEBUG_AUTH__ = DEBUG;

  log("auth helpers ready");
})();
