import { ref } from 'vue'

export function useFetch(url:string,options={}): { data: any; error: any } {
  const data = ref(null)
  const error = ref(null)

  fetch(url,options)
    .then((res) => res.json())
    .then((json) => (data.value = json))
    .catch((err) => (error.value = err))

  return { data, error }
}