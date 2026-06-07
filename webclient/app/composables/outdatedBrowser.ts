type BrowserName = "Opera" | "Edge" | "Chrome" | "Safari" | "Firefox";
interface BrowserInfo {
  name: BrowserName;
  version: number;
}

const BROWSER_NAME_REGEX = /(opera|chrome|safari|firefox(?=\/))\/?\s*(\d+)/i;
const OPERA_OR_EDGE_REGEX = /\b(OPR|Edge)\/(\d+)/;
const VERSION_OVERRIDE_REGEX = /version\/(\d+)/i;

function extractBrowserInfo(): BrowserInfo {
  const ua = navigator.userAgent;
  const M = ua.match(BROWSER_NAME_REGEX) || [];

  if (M[1] === "Chrome") {
    const operaOrEdge = ua.match(OPERA_OR_EDGE_REGEX);
    if (operaOrEdge != null) {
      const name = (operaOrEdge[1]?.replace("OPR", "Opera") ?? "Edge") as BrowserName;
      return { name, version: Number(operaOrEdge[2] ?? 0) };
    }
  }

  const match = M[2] ? [M[1], M[2]] : ["Netscape", navigator.appVersion];

  const versionOveride = ua.match(VERSION_OVERRIDE_REGEX);
  if (versionOveride != null) {
    match[1] = versionOveride[1] ?? "0";
  }

  return { name: match[0] as BrowserName, version: Number(match[1] ?? 0) };
}

function isSupportedBrowser(browserName: BrowserName, browserVersion: number) {
  switch (browserName) {
    case "Chrome":
      return browserVersion >= 98;
    case "Firefox":
      return browserVersion >= 94;
    case "Edge":
      return browserVersion >= 98;
    case "Opera":
      return browserVersion >= 84;
    case "Safari":
      return browserVersion >= 15.3;
    default:
      return false;
  }
}

function shouldWarnForOutdatedBrowser(): boolean {
  const browser = extractBrowserInfo();
  if (isSupportedBrowser(browser.name, browser.version)) return false;
  const optLastTime = localStorage.getItem("lastOutdatedBrowserWarningTime");
  if (optLastTime === null) return true;
  const lastTime = Number.parseInt(optLastTime, 10);

  const currentTime = Date.now();
  const msSinceLastWarning = currentTime - lastTime;
  const daysSinceLastWarning = msSinceLastWarning / (1000 * 60 * 60 * 24);
  return daysSinceLastWarning > 1;
}

if (shouldWarnForOutdatedBrowser()) {
  alert(
    `Please consider upgrading your browser to one of the following recommended options:

- Google Chrome
- Mozilla Firefox
- Microsoft Edge

We regret to inform you that your current web browser is outdated and unsupported for the optimal performance of this website.
To ensure a secure and efficient browsing experience, we recommend updating your browser to the latest version or switching to a more modern and supported browser.
Outdated browsers may not be able to render the website correctly thus leading to reduced functionality or lead to potential security vulnerabilities.

If you need assistance with updating your browser, please refer to your browser's official website or your IT department for guidance.
Thank you for your understanding, and we look forward to providing you with an enhanced browsing experience once your browser is up-to-date.`
  );
  localStorage.setItem("lastOutdatedBrowserWarningTime", Date.now().toString());
}

export {};
