describe("Check if navigating from the frontpage works as expected", () => {
  it("navigating to the mi", () => {
    cy.intercept("GET", "/api/get/root", { statusCode: 200, fixture: "get/root.json" });
    cy.visit("http://localhost:8000/");
    cy.intercept("GET", "/api/get/mi", { statusCode: 200, fixture: "get/mi.json" });
    cy.contains("Informatik").click();
    cy.url().should("include", "/building/mi");
  });
  it("navigating to an initally hidden entry", () => {
    cy.intercept("GET", "/api/get/root", { statusCode: 200, fixture: "get/root.json" });
    cy.visit("http://localhost:8000/");
    cy.contains("mehr").click();
    cy.intercept("GET", "/api/get/garching-interims", { statusCode: 200, fixture: "get/garching-interims.json" });
    cy.contains("Interims").click();
    cy.url().should("include", "/site/");
  });
  it("navigate to an campus", () => {
    cy.intercept("GET", "/api/get/root", { statusCode: 200, fixture: "get/root.json" });
    cy.visit("http://localhost:8000/");
    cy.intercept("GET", "/api/get/garching", { statusCode: 200, fixture: "get/garching.json" });
    cy.contains("Garching Forschungszentrum").click({ scrollBehavior: false });
    cy.url().should("include", "/campus/garching");
  });
});

export {};
