import AppFooter from "./AppFooter.vue";

describe("<AppFooter />", () => {
  it("the tum logo exists at all viewports", () => {
    cy.mount(AppFooter);
    cy.get("footer").should("exist");
    cy.viewport(300, 500);
    cy.contains("Offizieller Roomfinder").should("be.visible");
    cy.get("img").should("be.visible");
    cy.viewport(500, 500);
    cy.contains("Offizieller Roomfinder").should("be.visible");
    cy.get("img").should("be.visible");
    cy.viewport(700, 500);
    cy.contains("Offizieller Roomfinder").should("be.visible");
    cy.get("img").should("be.visible");
  });
});
