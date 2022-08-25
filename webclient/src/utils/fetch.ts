import { ref } from "vue";

export function useFetch<T>(
  url: string,
  options: RequestInit = {},
  successHandler = (d: T) => {}
) {
  const data = ref<T | null>(null);
  const error = ref<string | null>(null);

  // for some of our endpoints, we might want to have access to the lang/theme cookies
  options.credentials = "same-origin";

  fetch(url, options)
    .then((res) => res.json())
    .then((json) => {
      data.value = json;
      successHandler(json);
    })
    .catch((err) => (error.value = err));

  return { data, error };
}
