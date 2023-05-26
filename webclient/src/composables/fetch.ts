import { shallowRef } from "vue";
import { useGlobalStore } from "@/stores/global";

export function useFetch<T>(url: string, successHandler: (d: T) => void, options: RequestInit = {}) {
  const data = shallowRef<T | null>(null);
  // for some of our endpoints, we might want to have access to the lang/theme cookies

  // Add language query param to the request
  const lang = document.documentElement.lang;
  url += (url.indexOf("?") != -1 ? "&lang=" : "?lang=") + lang;

  const global = useGlobalStore();
  fetch(url, options)
    .then((res) => {
      if (res.status < 200 || res.status >= 300) throw res.statusText;
      return res.json();
    })
    .then((json) => {
      if (global.error_message) global.error_message = null;
      data.value = json;
      successHandler(json);
    })
    .catch((err) => (global.error_message = err));

  return { data };
}
