export type PreferencesSection = "allergens";

// Shared open-state so controls far from `PreferencesPopup` (e.g. the menu's price dropdown) can
// open it on a given section. One popup is mounted per layout, so a shared flag is safe.
export const usePreferencesPopup = () => {
  const isOpen = useState<boolean>("preferences-popup-open", () => false);
  const pendingSection = useState<PreferencesSection | null>(
    "preferences-popup-section",
    () => null
  );

  function open(section: PreferencesSection | null = null) {
    pendingSection.value = section;
    isOpen.value = true;
  }

  return { isOpen, pendingSection, open };
};
