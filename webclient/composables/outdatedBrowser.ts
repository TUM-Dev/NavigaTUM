type BrowserName = "Opera" | "Edge" | "Chrome" | "Safari" | "Firefox";
type BrowserInfo = {
  name: BrowserName;
  version: number;
};

function extractBrowserInfo(): BrowserInfo {
  const ua = navigator.userAgent;
  let tem;
  let M = ua.match(/(opera|chrome|safari|firefox(?=\/))\/?\s*(\d+)/i) || [];

  if (M[1] === "Chrome") {
    tem = ua.match(/\b(OPR|Edge)\/(\d+)/);
    if (tem != null) {
      return { name: tem[1].replace("OPR", "Opera") as BrowserName, version: +tem[2] };
    }
  }

  M = M[2] ? [M[1], M[2]] : [navigator.appName, navigator.appVersion, "-?"];

  if ((tem = ua.match(/version\/(\d+)/i)) != null) {
    M.splice(1, 1, tem[1]);
  }

  return { name: M[0] as BrowserName, version: +M[1] };
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
