import { ref } from "vue";

export function useFetch<T>(
  url: string,
  options = {},
  successHandler = (d: T) => {}
) {
  const data = ref<T | null>(null);
  const error = ref<string | null>(null);

  fetch(url, options)
    .then((res) => res.json())
    .then((json) => {
      data.value = json;
      successHandler(json);
    })
    .catch((err) => (error.value = err));

  return { data, error };
}
