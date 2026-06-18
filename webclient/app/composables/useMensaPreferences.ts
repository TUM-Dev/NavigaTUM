import { useLocalStorage } from "@vueuse/core";
import { isMensaPriceRole, type MensaPriceRole } from "~/utils/mensaMenu";

const DEFAULT_PRICE_ROLE: MensaPriceRole = "students";

/**
 * The visitor's preferred canteen price role, used by the menu card to decide which price to show.
 *
 * `useLocalStorage` gives persistence, reactivity, and same-tab sync across every consumer in one
 * primitive, so the on-card toggle and the settings popup - which can be mounted at the same time -
 * update each other live. It is `localStorage` rather than a cookie because the role only ever
 * drives client-side rendering (the menu is fetched with `server: false`), so it never needs to
 * reach the server. The serializer coerces tampered or legacy values back to the default.
 */
export const useMensaPreferences = () => {
  const priceRole = useLocalStorage<MensaPriceRole>("mensa-price-role", DEFAULT_PRICE_ROLE, {
    serializer: {
      read: (value) => (isMensaPriceRole(value) ? value : DEFAULT_PRICE_ROLE),
      write: (value) => value,
    },
  });

  return { priceRole };
};
