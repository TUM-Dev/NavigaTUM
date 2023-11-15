import AppFooter from "./AppFooter.vue";

describe("<AppFooter />", () => {
  it("renders", () => {
    // see: https://on.cypress.io/mounting-vue
    cy.mount(AppFooter);
  });
});
