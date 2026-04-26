type KnownUsage = {
  name_de: string;
  name_en: string;
  din_277: string;
  occurrences: number;
};

export type UsageOption = {
  /** slug used in the meilisearch index (lowercased, German name with umlauts preserved) */
  slug: string;
  /** localized display name */
  label: string;
  /** the other-language name, used for searching */
  altLabel: string;
  /** DIN 277 classification, e.g. "NF2.3" */
  din: string;
  occurrences: number;
};

function slugify(value: string): string {
  return value
    .replace(/[^a-zA-Z0-9\-äöüß.]+/g, "-")
    .toLowerCase()
    .replace(/^-+|-+$/g, "");
}

export function useKnownUsages() {
  const { locale } = useI18n();
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

  return {
    options,
    pending,
    error,
    refresh,
    labelFor,
  };
}
