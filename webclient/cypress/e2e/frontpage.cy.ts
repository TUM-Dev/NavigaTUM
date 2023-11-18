describe("Check if navigating from the frontpage works as expected", () => {
  it("navigating to the mri", () => {
    cy.intercept("GET", "/api/get/root?lang=de", { fixture: "get/root.de.json" });
    cy.visit("http://localhost:3000/");
    cy.intercept("GET", "/api/get/mi?lang=de", { fixture: "get/mi.de.json" });
    cy.contains("MRI").click();
    cy.url().should("to.match", /(site|campus|view)\/mri/);
  });
  it("navigating to an initally hidden entry", () => {
    cy.intercept("GET", "/api/get/root?lang=de", { fixture: "get/root.de.json" });
    cy.visit("http://localhost:3000/");
    cy.contains("mehr").click();
    cy.intercept("GET", "/api/get/garching-interims?lang=de", { fixture: "get/garching-interims.de.json" });
    cy.contains("Interims").click();
    cy.url().should("include", "/site/");
  });
  it("navigate to an campus", () => {
    cy.intercept("GET", "/api/get/root?lang=de", { fixture: "get/root.de.json" });
    cy.visit("http://localhost:3000/");
    cy.intercept("GET", "/api/get/garching?lang=de", { fixture: "get/garching.de.json" });
    cy.contains("Garching Forschungszentrum").click({ scrollBehavior: false });
    cy.url().should("include", "/campus/garching");
  });
});

export {};
