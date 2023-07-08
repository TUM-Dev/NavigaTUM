import type { Path } from "@intlify/core-base";
import type { TranslateOptions, DateTimeOptions, IsNever, IsEmptyObject, PickupPaths } from "@intlify/core-base";
import type { DefineLocaleMessage, RemovedIndexResources } from "vue-i18n-core/src/composer";

declare module "vue" {
  export interface ComponentCustomProperties {
    /**
     * Datetime formatting
     *
     * @param value - A value, timestamp number or `Date` instance
     * @param options - An options, see the {@link DateTimeOptions}
     *
     * @returns formatted value
     */
    $d(value: number | Date, options?: DateTimeOptions): string;

    /**
     * Locale message translation
     *
     * @param key - A target locale message key
     * @param plural - Which plural string to get. 1 returns the first one.
     * @param options - An options, see the {@link TranslateOptions}
     *
     * @returns translation message
     */
    $t<
      Key extends string,
      DefinedLocaleMessage extends
        RemovedIndexResources<DefineLocaleMessage> = RemovedIndexResources<DefineLocaleMessage>,
      Keys = IsEmptyObject<DefinedLocaleMessage> extends false
        ? PickupPaths<{
            [K in keyof DefinedLocaleMessage]: DefinedLocaleMessage[K];
          }>
        : never,
      ResourceKeys extends Keys = IsNever<Keys> extends false ? Keys : never,
    >(
      key: Key | ResourceKeys | Path,
      plural?: number,
      options?: TranslateOptions,
    ): string;
  }
}

export {};
