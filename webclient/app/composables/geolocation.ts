export interface GeolocationState {
  shouldTriggerMapGeolocation: boolean;
  mapGeolocationActive: boolean;
  triggeringSearchBarId: string | null;
  userLocation: { lat: number; lon: number } | null;
}

export const useSharedGeolocation = () =>
  useState<GeolocationState>("geolocation", () => ({
    shouldTriggerMapGeolocation: false,
    mapGeolocationActive: false,
    triggeringSearchBarId: null,
    userLocation: null,
  }));
