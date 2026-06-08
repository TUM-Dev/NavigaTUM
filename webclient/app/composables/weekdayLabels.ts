import type { OpeningHoursWeekday } from "~/utils/openingHoursEditor";

// Localized weekday names for the opening-hours schedule editors, in one place
// so WeekScheduleInput and SemesterScheduleInput never drift apart. Uses its own
// local i18n scope (the same mechanism an SFC `<i18n>` block compiles to), so the
// labels track the active locale without polluting the global message catalog.
export function useWeekdayLabels() {
  const { t } = useI18n({
    useScope: "local",
    messages: {
      de: {
        Mo: "Montag",
        Tu: "Dienstag",
        We: "Mittwoch",
        Th: "Donnerstag",
        Fr: "Freitag",
        Sa: "Samstag",
        Su: "Sonntag",
      },
      en: {
        Mo: "Monday",
        Tu: "Tuesday",
        We: "Wednesday",
        Th: "Thursday",
        Fr: "Friday",
        Sa: "Saturday",
        Su: "Sunday",
      },
    },
  });
  return computed<Record<OpeningHoursWeekday, string>>(() => ({
    Mo: t("Mo"),
    Tu: t("Tu"),
    We: t("We"),
    Th: t("Th"),
    Fr: t("Fr"),
    Sa: t("Sa"),
    Su: t("Su"),
  }));
}
