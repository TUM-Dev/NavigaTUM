function extractBrowserInfo() {
    let ua = navigator.userAgent,
        tem,
        M =
            ua.match(
                /(opera|chrome|safari|firefox|msie|trident(?=\/))\/?\s*(\d+)/i
            ) || [];

    if (/trident/i.test(M[1])) {
        tem = /\brv[ :]+(\d+)/g.exec(ua) || [];
        return {name: "IE", version: tem[1] || ""};
    }

    if (M[1] === "Chrome") {
        tem = ua.match(/\b(OPR|Edge)\/(\d+)/);
        if (tem != null) {
            return {name: tem[1].replace("OPR", "Opera"), version: tem[2]};
        }
    }

    M = M[2] ? [M[1], M[2]] : [navigator.appName, navigator.appVersion, "-?"];

    if ((tem = ua.match(/version\/(\d+)/i)) != null) {
        M.splice(1, 1, tem[1]);
    }

    return {name: M[0], version: M[1]};
}

const minSupportedBrowsers = {
      Chrome: 98,
      Firefox: 94,
      Edge: 98,
      Opera: 84,
      Safari: 15.3,
};

function isUnSupportedBrowser() {
    const browser = extractBrowserInfo();
    const browserNameIsKnown = minSupportedBrowsers.hasOwnProperty(browser.name);
    return browserNameIsKnown && (+browser.version < minSupportedBrowsers[browser.name]);
}

function shouldWarn(){
    if (!isUnSupportedBrowser()) return false;
    const lastTime = localStorage.getItem("lastOutdatedBrowserWarningTime");
    if (lastTime===null) return true;

    const currentTime = new Date().getTime();
    const daysSinceLastWarning = (currentTime - Date(lastTime)) / (1000 * 60 * 60 * 24);
    return daysSinceLastWarning > 1;
}

if (shouldWarn()){
    const parent = document.getElementById("errorToasts");
    const error = document.createElement("div");
    error.classList.add("toast", "toast-error");
    error.innerHTML = "${{_.core_js.error.browser_outdated}}$";
    parent.appendChild(error);
    localStorage.setItem("lastOutdatedBrowserWarningTime", new Date().getTime().toString());
}