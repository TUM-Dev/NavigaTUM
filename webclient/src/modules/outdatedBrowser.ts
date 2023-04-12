function extractBrowserInfo() {
  const ua = navigator.userAgent;
  let tem;
  let M = ua.match(/(opera|chrome|safari|firefox(?=\/))\/?\s*(\d+)/i) || [];

  if (M[1] === "Chrome") {
    tem = ua.match(/\b(OPR|Edge)\/(\d+)/);
    if (tem != null) {
      return { name: tem[1].replace("OPR", "Opera"), version: +tem[2] };
    }
  }

  M = M[2] ? [M[1], M[2]] : [navigator.appName, navigator.appVersion, "-?"];

  if ((tem = ua.match(/version\/(\d+)/i)) != null) {
    M.splice(1, 1, tem[1]);
  }

  return { name: M[0], version: +M[1] };
}

function isSupportedBrowser(browserName: string, browserVersion: number) {
  switch (browserName) {
    case "Chrome":
      return 98 >= browserVersion;
    case "Firefox":
      return 94 >= browserVersion;
    case "Edge":
      return 98 >= browserVersion;
    case "Opera":
      return 84 >= browserVersion;
    case "Safari":
      return 15.3 >= browserVersion;
    default:
      return false;
  }
}

function shouldWarnForOutdatedBrowser() {
  const browser = extractBrowserInfo();
  if (isSupportedBrowser(browser.name, browser.version)) return false;
  const optLastTime = localStorage.getItem("lastOutdatedBrowserWarningTime");
  if (optLastTime === null) return true;
  const lastTime = new Date(optLastTime);

  const currentTime: Date = new Date();
  const msSinceLastWarning = currentTime.getTime() - lastTime.getTime();
  const daysSinceLastWarning = msSinceLastWarning / (1000 * 60 * 60 * 24);
  return daysSinceLastWarning > 1;
}

if (shouldWarnForOutdatedBrowser()) {
  const error = document.createElement("div");
  error.classList.add("toast", "toast-error");
  error.innerHTML = "${{_.core_js.error.browser_outdated}}$";
  document.getElementById("errorToasts")?.appendChild(error);
  localStorage.setItem("lastOutdatedBrowserWarningTime", new Date().getTime().toString());
}
export {};
