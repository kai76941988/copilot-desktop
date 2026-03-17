// OAuth and Authentication Logic

// Check if URL matches OAuth/authentication patterns
function matchesAuthUrl(url, baseUrl = window.location.href) {
  try {
    const urlObj = new URL(url, baseUrl);
    const hostname = urlObj.hostname.toLowerCase();
    const pathname = urlObj.pathname.toLowerCase();
    const fullUrl = urlObj.href.toLowerCase();

    // Common OAuth providers and paths
    const oauthPatterns = [
      /accounts\.google\.com/,
      /accounts\.google\.[a-z]+/,
      /microsoftonline\.[a-z.]+/,
      /microsoftonline-p\.com/,
      /login\.microsoft\.com/,
      /login\.windows\.net/,
      /login\.live\.com/,
      /account\.live\.com/,
      /msauth\.net/,
      /msftauth\.net/,
      /msauthimages\.net/,
      /msftauthimages\.net/,
      /aadcdn\.msauth\.net/,
      /aadcdn\.msftauth\.net/,
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

    const isMatch = oauthPatterns.some(
      (pattern) =>
        pattern.test(hostname) ||
        pattern.test(pathname) ||
        pattern.test(fullUrl),
    );

    if (isMatch) {
      console.log("[Pake] OAuth URL detected:", url);
    }

    return isMatch;
  } catch (e) {
    return false;
  }
}

// Check if URL is an OAuth/authentication link
function isAuthLink(url) {
  return matchesAuthUrl(url);
}

// Check if this is an OAuth/authentication popup
function isAuthPopup(url, name) {
  // Check for known authentication window names
  const authWindowNames = [
    "appleauthentication",
    "oauth2",
    "oauth",
    "google-auth",
    "auth-popup",
    "signin",
    "login",
    "msal",
    "msalv2",
    "microsoft",
    "aad",
    "msauth",
    "microsoftauth",
    "wlid",
    "live",
  ];

  const normalizedName = (name || "").toLowerCase();
  if (
    normalizedName &&
    authWindowNames.some((item) => normalizedName.includes(item))
  ) {
    return true;
  }

  return matchesAuthUrl(url);
}

// Export functions to global scope
window.matchesAuthUrl = matchesAuthUrl;
window.isAuthLink = isAuthLink;
window.isAuthPopup = isAuthPopup;
