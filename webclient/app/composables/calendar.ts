type CalendarState = {
  open: boolean;
  showing: string[];
};
export const useCalendar = () =>
  useState<CalendarState>("calendar", () => ({
    open: false,
    showing: [],
  }));
