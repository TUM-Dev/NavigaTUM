import { ref } from "vue";

export function useFetch<T>(
  url: string,
  options: RequestInit = {},
  successHandler = (d: T) => {}
) {
  const data = ref<T | null>(null);
  const error = ref<string | null>(null);

  // we can only add the lang cookie if
  // - the url is local and thus accepting certificates or
  // - we are in production, and thus we are the same origin
  if (import.meta.env.PROD) options.credentials = "same-origin";
  else if (import.meta.env.API_URL?.startsWith("http://localhost"))
    options.credentials = "include";

  fetch(url, options)
    .then((res) => res.json())
    .then((json) => {
      data.value = json;
      successHandler(json);
    })
    .catch((err) => (error.value = err));

  return { data, error };
}
