describe("Check if submitting feedback works as expected", () => {
  it("main page", () => {
    cy.visit("http://localhost:8000/");
    cy.contains("Standorte");

    cy.get('[data-cy="main-footer"]').scrollIntoView();
    checkFeedbackForm('[data-cy="open-feedback-footer"]');
  });
  it("search page", () => {
    cy.visit("http://localhost:8000/search?q=mw%20fachschaft");

    checkFeedbackForm('[data-cy="open-feedback-search"]');
  });
  it("details page (general feedback)", () => {
    cy.visit("http://localhost:8000/view/mi");
    cy.get('[data-cy="open-feedback-details"]').should("exist", { timeout: 10_000 }); // wait for the site to be interactive

    checkFeedbackForm('[data-cy="open-feedback-details"]');
  });
});

function checkFeedbackForm(selector_which_should_open_the_modal: string) {
  // open the modal
  cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback_token.json" });
  cy.get("#feedback-modal").should("not.exist");
  cy.get(selector_which_should_open_the_modal).click({ scrollBehavior: false });
  // check that the modal is opened
  cy.get("#feedback-modal").should("exist");
  cy.get("#feedback-error").should("be.empty");
  cy.get("#feedback-privacy").should("not.be.checked");
  cy.get("#feedback-delete").should("not.be.checked");
  // make shure that the modal is empty
  cy.get("#feedback-subject").clear();
  cy.get("#feedback-body").clear();
  // try to submit without filling out the form
  cy.get("#feedback-send").click();
  cy.get("#feedback-error").contains("Betreff fehlt");

  // fill out the form partially
  cy.get("#feedback-subject").type("A catchy title");
  cy.get("#feedback-send").click();
  cy.get("#feedback-error").contains("Nachricht fehlt");

  // fill out the form, but don't accept the privacy policy
  cy.get("#feedback-body").type("A clear description what happened where and how we should improve it");
  cy.get("#feedback-send").click();
  cy.get("#feedback-error").contains("Datenschutz");

  // accept the privacy policy
  cy.get("#feedback-privacy").parent().click();
  cy.get("#feedback-modal .modal-body").scrollTo("bottom");
  cy.intercept("POST", "/api/feedback/feedback", { statusCode: 201, fixture: "feedback_response.json" });
  cy.get("#feedback-send").click();

  // check that the next page is loaded correctly
  cy.get("#feedback-modal", { timeout: 10_000 }).should("not.exist"); // wait for the site to be interactive
  cy.get("#feedback-success-modal").should("exist");
  cy.contains("Vielen Dank für dein Feedback!");
  cy.contains("OK").click();
  cy.get("#feedback-success-modal").should("not.exist");
}

export {};
