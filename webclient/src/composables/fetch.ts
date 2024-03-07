import { type MaybeRefOrGetter, shallowRef, toValue, watchEffect } from "vue";
import { useGlobalStore } from "@/stores/global";

export function useFetch<T>(
  url: MaybeRefOrGetter<string>,
  successHandler: (d: T) => void,
  errorHandler: ((e: string) => void) | undefined = undefined,
) {
  const data = shallowRef<T | null>(null);
  const fetchData = () => {
    // for some of our endpoints, we might want to have access to the lang/theme cookies
    // Add language query param to the request
    let fetchUrl = toValue(url);
    const lang = localStorage.getItem("lang") || "de";
    const langQuery = `${fetchUrl.indexOf("?") != -1 ? "&lang=" : "?lang="}${lang}`;
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
