import { ref } from "vue";

export function useFetch<T>(
  url: string,
  options: RequestInit = {},
  successHandler = (d: T) => {}
) {
  const data = ref<T | null>(null);
  const error = ref<string | null>(null);

  options.credentials = import.meta.env.PROD ? "same-origin" : "include";

  fetch(url, options)
    .then((res) => res.json())
    .then((json) => {
      data.value = json;
      successHandler(json);
    })
    .catch((err) => (error.value = err));

  return { data, error };
}
