import type { Router } from "vue-router";
import type { Component } from "vue";
import type { VueTestUtils } from "cypress/vue";
import { ComponentPublicInstance } from "vue";

type Options = Pick<typeof VueTestUtils, "config">["config"] & { router?: Router };
export type OptionsParam = Options | Record<string, never>;
declare global {
  namespace Cypress {
    interface Chainable {
      mount<Props extends NonNullable<unknown>>(
        component: Component,
        options?: OptionsParam,
      ): Cypress.Chainable<{
        wrapper: VueWrapper<ComponentPublicInstance<Props>>;
        component: VueWrapper<ComponentPublicInstance<Props>>["vm"];
      }>;
    }
  }
}
