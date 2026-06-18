import { useLocalStorage } from "@vueuse/core";
import { isMensaPriceRole, isSelectableAllergen, type MensaPriceRole } from "~/utils/mensaMenu";

const DEFAULT_PRICE_ROLE: MensaPriceRole = "students";

// `localStorage`, not a cookie: these only drive client-side rendering and never reach the server.
// The serializers coerce tampered or legacy values back to a safe default.
export const useMensaPreferences = () => {
  const priceRole = useLocalStorage<MensaPriceRole>("mensa-price-role", DEFAULT_PRICE_ROLE, {
    serializer: {
      read: (value) => (isMensaPriceRole(value) ? value : DEFAULT_PRICE_ROLE),
      write: (value) => value,
    },
  });

  const allergenWarnings = useLocalStorage<string[]>("mensa-allergen-warnings", [], {
    serializer: {
      read: (value) => {
        try {
          const parsed: unknown = JSON.parse(value);
          if (!Array.isArray(parsed)) return [];
          return parsed.filter(
            (entry): entry is string => typeof entry === "string" && isSelectableAllergen(entry)
          );
        } catch {
          return [];
        }
      },
      write: (value) => JSON.stringify(value),
    },
  });

  return { priceRole, allergenWarnings };
};
