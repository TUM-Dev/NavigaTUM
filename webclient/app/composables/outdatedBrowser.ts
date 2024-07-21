type BrowserName = "Opera" | "Edge" | "Chrome" | "Safari" | "Firefox";
type BrowserInfo = {
  name: BrowserName;
  version: number;
};

function extractBrowserInfo(): BrowserInfo {
  const ua = navigator.userAgent;
  let M = ua.match(/(opera|chrome|safari|firefox(?=\/))\/?\s*(\d+)/i) || [];

  if (M[1] === "Chrome") {
    let operaOrEdge = ua.match(/\b(OPR|Edge)\/(\d+)/);
    if (operaOrEdge != null) {
      const name = (operaOrEdge[1]?.replace("OPR", "Opera") ?? "Edge") as BrowserName;
      return { name, version: +(operaOrEdge[2] ?? 0) };
    }
  }

  let match = M[2] ? [M[1], M[2]] : ["Netscape", navigator.appVersion];

  const versionOveride = ua.match(/version\/(\d+)/i);
  if (versionOveride != null) {
    match[1] = versionOveride[1] ?? "0";
  }

  return { name: match[0] as BrowserName, version: +(match[1] ?? 0) };
}

function isSupportedBrowser(browserName: BrowserName, browserVersion: number) {
  switch (browserName) {
    case "Chrome":
      return 98 <= browserVersion;
    case "Firefox":
      return 94 <= browserVersion;
    case "Edge":
      return 98 <= browserVersion;
    case "Opera":
      return 84 <= browserVersion;
    case "Safari":
      return 15.3 <= browserVersion;
    default:
      return false;
  }
}

function shouldWarnForOutdatedBrowser(): boolean {
  const browser = extractBrowserInfo();
  if (isSupportedBrowser(browser.name, browser.version)) return false;
  console.table(browser);
  const optLastTime = localStorage.getItem("lastOutdatedBrowserWarningTime");
  if (optLastTime === null) return true;
  const lastTime = new Date(optLastTime);

  const currentTime: Date = new Date();
  const msSinceLastWarning = currentTime.getTime() - lastTime.getTime();
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
Thank you for your understanding, and we look forward to providing you with an enhanced browsing experience once your browser is up-to-date.`,
  );
  localStorage.setItem("lastOutdatedBrowserWarningTime", new Date().getTime().toString());
}
export {};
