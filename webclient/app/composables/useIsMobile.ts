import { useMediaQuery } from "@vueuse/core";

// Matches the breakpoint at which our MapLibre map controls switch to a
// compact / fullscreen layout. Kept as a single source of truth so the
// magic string isn't duplicated across map components.
export function useIsMobile() {
  return useMediaQuery("only screen and (max-width: 480px)");
}
