import type { mount } from "cypress/vue";
declare global {
  namespace Cypress {
    interface Chainable {
      mount: mount;
    }
  }
}
