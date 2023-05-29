describe("Check if navigating from the frontpage works as expected", () => {
  it("navigating to the mi", () => {
    cy.visit("http://localhost:8000/");
    cy.contains("Informatik").click();
    cy.url().should("include", "/building/mi");
  });
  it("navigating to an initally hidden entry", () => {
    cy.visit("http://localhost:8000/");
    cy.contains("mehr").click();
    cy.contains("Interims").click();
    cy.url().should("include", "/site/garching-interims");
  });
  it("navigate to an campus", () => {
    cy.visit("http://localhost:8000/");
    cy.contains("Garching Forschungszentrum").click({ scrollBehavior: false });
    cy.url().should("include", "/campus/garching");
  });
});

export {};
