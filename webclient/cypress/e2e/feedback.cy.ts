describe("Check if opening the feedback form works from every subview", () => {
  it("main page", () => {
    cy.intercept("GET", "/api/get/root?lang=de", { fixture: "get/root.de.json" });
    cy.visit("http://localhost:3000/");
    cy.contains("Standorte");

    cy.get('[data-cy="main-footer"]').scrollIntoView();
    checkFeedbackForm('[data-cy="open-feedback-footer"]');
  });
  it("search page", () => {
    cy.intercept("GET", "/api/search?q=fsmb&limit_buildings=10&limit_rooms=30&limit_all=30&lang=de", {
      fixture: "search/fsmb.long.de.json",
    });
    cy.visit("http://localhost:3000/search?q=fsmb");

    checkFeedbackForm('[data-cy="open-feedback-search"]');
  });
  it("details page (general feedback)", () => {
    cy.intercept("GET", "/api/get/mi?lang=de", { fixture: "get/mi.de.json" });
    cy.visit("http://localhost:3000/view/mi");
    cy.get('[data-cy="open-feedback-details"]').should("exist", { timeout: 10_000 }); // wait for the site to be interactive

    checkFeedbackForm('[data-cy="open-feedback-details"]');
  });
});

function checkFeedbackForm(selector_which_should_open_the_modal: string) {
  cy.get('[data-cy="feedback-modal"]').should("not.exist");
  cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
  cy.get(selector_which_should_open_the_modal).click({ scrollBehavior: false });
  cy.get('[data-cy="feedback-modal"]').should("exist");
}

export {};
