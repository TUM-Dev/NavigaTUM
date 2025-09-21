<script setup lang="ts">
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div class="relative">
    {{ t("no_webgl.no_browser_support") }}
    {{ t("no_webgl.explain_webgl") }} <br />
    {{ t("no_webgl.please_try") }}:
    <ol>
      <li>
        {{ t("no_webgl.upgrade_browser") }}
        {{ t("no_webgl.visit_official_website_to_upgrade_browser") }}
      </li>
      <li>
        {{ t("no_webgl.try_different_browser") }}
        {{ t("no_webgl.known_good_browsers") }}
        {{ t("no_webgl.try_different_browser2") }}
      </li>
    </ol>
  </div>
</template>

<i18n lang="yaml">
de:
  no_webgl:
    explain_webgl: WebGL ist eine Technologie, die interaktive 3D-Grafiken direkt im Webbrowser ermöglicht, und leider scheint Ihr Browser diese Fähigkeit nicht zu besitzen.
    known_good_browsers: Beliebte Browser wie Google Chrome, Mozilla Firefox und Microsoft Edge sind dafür bekannt, dass sie WebGL-Funktionen haben.
    no_browser_support: Es tut uns leid, aber es scheint, dass Ihr Browser WebGL nicht unterstützt, was für die Anzeige der Karte erforderlich ist.
    please_try: Um dieses Problem zu beheben und die Karte anzuzeigen, empfehlen wir, eine der folgenden Möglichkeiten auszuprobieren
    try_different_browser: Alternativ kannst du auch einen anderen Browser verwenden, der WebGL unterstützt.
    try_different_browser2: Versuche, einen dieser Browser zu installieren und die Karte erneut aufzurufen.
    upgrade_browser: Aktualisiere deinen aktuellen Browser auf die neueste Version, da neuere Versionen oft Unterstützung für WebGL enthalten.
    visit_official_website_to_upgrade_browser: Du kannst die offizielle Website Ihres Browsers besuchen, um das neueste Update herunterzuladen und zu installieren.
en:
  no_webgl:
    explain_webgl: WebGL is a technology that enables interactive 3D graphics directly in the web browser, and unfortunately, your browser does not seem to have this capability.
    known_good_browsers: Popular browsers like Google Chrome, Mozilla Firefox, and Microsoft Edge are known to have WebGL capabilities.
    no_browser_support: We are sorry, but it seems that your browser does not support WebGL, which is required to display the map.
    please_try: To resolve this issue and view the map, we recommend trying one of the following
    try_different_browser: Alternatively, you can try using a different browser that supports WebGL.
    try_different_browser2: Consider installing one of these browsers and accessing the map again.
    upgrade_browser: Upgrade your current browser to the latest version, as newer versions often include support for WebGL.
    visit_official_website_to_upgrade_browser: You can visit the official website of your browser to download and install the latest update.
</i18n>
