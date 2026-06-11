interface KnownOrg {
  org_id: number;
  code: string;
  name_de: string;
  name_en: string;
}

export interface OrgOption {
  org_id: number;
  code: string;
  label: string;
  altLabel: string;
  nameDe: string;
  nameEn: string;
}

const MAX_RESULTS = 50;

const FETCH_KEY = "known-orgs";
const FETCH_PATH = "/cdn/known_orgs.json";

export function useKnownOrgs() {
  const { locale } = useI18n();
  const runtimeConfig = useRuntimeConfig();

  const handle = useFetch<KnownOrg[]>(`${runtimeConfig.public.cdnURL}${FETCH_PATH}`, {
    key: FETCH_KEY,
    server: true,
    lazy: true,
    default: () => [],
  });
  const { data, pending, error, refresh } = handle;

  const options = computed<OrgOption[]>(() => {
    const isDe = locale.value === "de";

    return (data.value ?? []).map((o) => ({
      org_id: o.org_id,
      code: o.code,
      label: isDe ? o.name_de : o.name_en,
      altLabel: isDe ? o.name_en : o.name_de,
      nameDe: o.name_de,
      nameEn: o.name_en,
    }));
  });

  function byId(id: number | null): OrgOption | null {
    if (id === null) return null;
    return options.value.find((o) => o.org_id === id) ?? null;
  }

  function filter(query: string): OrgOption[] {
    const q = query.trim().toLowerCase();
    const matches = q
      ? options.value.filter(
          (o) =>
            o.label.toLowerCase().includes(q) ||
            o.altLabel.toLowerCase().includes(q) ||
            o.code.toLowerCase().includes(q)
        )
      : options.value;
    return matches.slice(0, MAX_RESULTS);
  }

  return {
    options,
    pending,
    error,
    refresh,
    byId,
    filter,
    maxResults: MAX_RESULTS,
    // Awaitable underlying fetch. Resolves once the orgs CDN has settled, useful
    // at page setup so byId() can resolve a pre-set selection on first render.
    ready: () => handle,
  };
}
