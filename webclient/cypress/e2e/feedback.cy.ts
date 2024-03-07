describe("Check if opening the feedback form works from every subview", () => {
  it("main page", () => {
    cy.intercept("GET", "/api/get/root?lang=de", { fixture: "get/root.de.json" });
    cy.visit("http://localhost:3000/");
    cy.contains("Sites");

    cy.get("footer").scrollIntoView();
    cy.contains("Send Feedback").should("not.exist");
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
    cy.get("footer").contains("Feedback").click({ scrollBehavior: false });
    cy.contains("Send Feedback").should("exist");
  });
  it("search page", () => {
    cy.intercept("GET", "/api/search?q=fsmb&limit_buildings=10&limit_rooms=30&limit_all=30&lang=de", {
      fixture: "search/fsmb.long.de.json",
    });
    cy.visit("http://localhost:3000/search?q=fsmb");

    cy.contains("Send Feedback").should("not.exist");
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
    cy.get("footer").contains("Feedback").click();
    cy.contains("Send Feedback").should("exist");
  });
  it("details page (general feedback)", () => {
    cy.intercept("GET", "/api/get/mi?lang=de", { fixture: "get/mi.de.json" });
    cy.visit("http://localhost:3000/view/mi");
    cy.get('[data-cy="open-feedback-details"]').should("exist", { timeout: 10_000 }); // wait for the site to be interactive

    cy.contains("Send Feedback").should("not.exist");
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
    cy.get("footer").contains("Feedback").click();
    cy.contains("Send Feedback").should("exist");
  });
  it("details page (general feedback)", () => {
    cy.intercept("GET", "/api/get/mi?lang=de", { fixture: "get/mi.de.json" });
    cy.visit("http://localhost:3000/view/mi");
    cy.get('[data-cy="open-feedback-details"]').should("exist", { timeout: 10_000 }); // wait for the site to be interactive

    cy.contains("Send Feedback").should("not.exist");
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
    cy.get('[data-cy="open-feedback-details"]').click({ scrollBehavior: false });
    cy.contains("Send Feedback").should("exist");
  });
});

export {};
