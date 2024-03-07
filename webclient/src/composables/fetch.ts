import { type MaybeRefOrGetter, shallowRef, toValue, watchEffect } from "vue";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";

export function useFetch<T>(
  url: MaybeRefOrGetter<string>,
  successHandler: (d: T) => void,
  errorHandler: ((e: string) => void) | undefined = undefined,
) {
  const data = shallowRef<T | null>(null);
  const fetchData = () => {
    // for some of our endpoints, we might want to have access to the lang/theme cookies
    // Add language query param to the request
    const fetchUrl = toValue(url);
    const { locale } = useI18n({ useScope: "global" });
    const langQuery = `${fetchUrl.indexOf("?") != -1 ? "&lang=" : "?lang="}${locale}`;
    const localisedUrl = `${import.meta.env.VITE_APP_URL}${fetchUrl}${langQuery}`;

    const global = useGlobalStore();
    const fetchErrorHandler = errorHandler || ((err: string) => (global.error_message = err));
    fetch(localisedUrl)
      .then((res) => {
        if (res.status < 200 || res.status >= 300) throw res.statusText;
        return res.json();
      })
      .then((json) => {
        if (global.error_message) global.error_message = null;
        data.value = json;
        successHandler(json);
      })
      .catch(fetchErrorHandler);
  };
  watchEffect(() => {
    fetchData();
  });
  return { data };
}
