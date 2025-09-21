export interface GeolocationState {
  shouldTriggerMapGeolocation: boolean;
  mapGeolocationActive: boolean;
}

export const useSharedGeolocation = () =>
  useState<GeolocationState>("geolocation", () => ({
    shouldTriggerMapGeolocation: false,
    mapGeolocationActive: false,
  }));
