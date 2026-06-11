interface KnownUsage {
  usage_id: number;
  name_de: string;
  name_en: string;
  din_277: string;
  occurrences: number;
}

export interface UsageOption {
  /** numeric primary key matching `usages_tumonline.csv`; required when submitting an addition. */
  usage_id: number;
  /** slug used in the meilisearch index (lowercased, German name with umlauts preserved) */
  slug: string;
  /** localized display name */
  label: string;
  /** the other-language name, used for searching */
  altLabel: string;
  /** DIN 277 classification, e.g. "NF2.3" */
  din: string;
  occurrences: number;
}

function slugify(value: string): string {
  return value
    .replace(/[^a-zA-Z0-9\-äöüß.]+/g, "-")
    .toLowerCase()
    .replace(/^-+|-+$/g, "");
}

export function useKnownUsages() {
  // Explicit global scope so we don't collide with the consuming component's
  // own `useI18n({useScope: "local"})` (the bare call would default to 'local'
  // when the consumer has an <i18n> block).
  const { locale } = useI18n({ useScope: "global" });
  const runtimeConfig = useRuntimeConfig();

  const { data, pending, error, refresh } = useFetch<KnownUsage[]>(
    `${runtimeConfig.public.cdnURL}/cdn/known_usages.json`,
    {
      key: "known-usages",
      server: true,
      lazy: true,
      default: () => [],
    }
  );

  const options = computed<UsageOption[]>(() => {
    const isDe = locale.value === "de";

    return (data.value ?? []).map((u) => ({
      usage_id: u.usage_id,
      slug: slugify(u.name_de),
      label: isDe ? u.name_de : u.name_en,
      altLabel: isDe ? u.name_en : u.name_de,
      din: u.din_277,
      occurrences: u.occurrences,
    }));
  });

  function labelFor(slug: string): string {
    return options.value.find((o) => o.slug === slug)?.label ?? slug;
  }

  function byId(id: number | null): UsageOption | null {
    if (id === null) return null;
    return options.value.find((o) => o.usage_id === id) ?? null;
  }

  function filter(query: string): UsageOption[] {
    const q = query.trim().toLowerCase();
    if (!q) return options.value;
    return options.value.filter(
      (o) =>
        o.label.toLowerCase().includes(q) ||
        o.altLabel.toLowerCase().includes(q) ||
        o.din.toLowerCase().includes(q)
    );
  }

  return {
    options,
    pending,
    error,
    refresh,
    labelFor,
    byId,
    filter,
  };
}
