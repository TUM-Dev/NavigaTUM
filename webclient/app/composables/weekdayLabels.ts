import type { OpeningHoursWeekday } from "~/utils/openingHoursEditor";

const MESSAGES = {
  de: {
    weekdayLabels: {
      Mo: "Montag",
      Tu: "Dienstag",
      We: "Mittwoch",
      Th: "Donnerstag",
      Fr: "Freitag",
      Sa: "Samstag",
      Su: "Sonntag",
    },
  },
  en: {
    weekdayLabels: {
      Mo: "Monday",
      Tu: "Tuesday",
      We: "Wednesday",
      Th: "Thursday",
      Fr: "Friday",
      Sa: "Saturday",
      Su: "Sunday",
    },
  },
} as const;

let messagesMerged = false;

export function useWeekdayLabels() {
  const i18n = useI18n({ useScope: "global" });
  if (!messagesMerged) {
    i18n.mergeLocaleMessage("de", MESSAGES.de);
    i18n.mergeLocaleMessage("en", MESSAGES.en);
    messagesMerged = true;
  }
  const tt = (k: OpeningHoursWeekday) => i18n.t(`weekdayLabels.${k}`);
  return computed<Record<OpeningHoursWeekday, string>>(() => ({
    Mo: tt("Mo"),
    Tu: tt("Tu"),
    We: tt("We"),
    Th: tt("Th"),
    Fr: tt("Fr"),
    Sa: tt("Sa"),
    Su: tt("Su"),
  }));
}
