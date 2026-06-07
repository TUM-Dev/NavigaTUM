interface KnownOrg {
  org_id: number;
  code: string;
  name_de: string;
  name_en: string;
}

export interface OrgOption {
  /** numeric primary key submitted as `events.organising_org_id`; required for an event addition. */
  org_id: number;
  /** TUMonline org code, the disambiguator for the ~100 orgs that share a localized name. */
  code: string;
  /** localized display name */
  label: string;
  /** the other-language name, used for searching */
  altLabel: string;
}

// The org tree has ~2.3k entries; rendering them all into the combobox on an empty query is
// needless. Cap the visible matches and let the user narrow by typing.
const MAX_RESULTS = 50;

export function useKnownOrgs() {
  const { locale } = useI18n();
  const runtimeConfig = useRuntimeConfig();

  const { data, pending, error, refresh } = useFetch<KnownOrg[]>(
    `${runtimeConfig.public.cdnURL}/cdn/known_orgs.json`,
    {
      key: "known-orgs",
      server: true,
      lazy: true,
      default: () => [],
    }
  );

  const options = computed<OrgOption[]>(() => {
    const isDe = locale.value === "de";

    return (data.value ?? []).map((o) => ({
      org_id: o.org_id,
      code: o.code,
      label: isDe ? o.name_de : o.name_en,
      altLabel: isDe ? o.name_en : o.name_de,
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
  };
}
