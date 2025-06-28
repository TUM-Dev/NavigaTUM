import { useRouteQuery } from "@vueuse/router";

export const useCalendar = () =>
  useRouteQuery<string[]>("calendar[]", [], {
    transform: (val) => {
      // type cohersion here is really fucked val in reallity can be null, string, string[]
      if (val === null) return [] as string[];
      if (Array.isArray(val)) return [...new Set(val)];
      return [val];
    },
  });
