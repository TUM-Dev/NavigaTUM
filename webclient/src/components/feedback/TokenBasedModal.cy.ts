import TokenBasedModal from "./TokenBasedModal.vue";

describe("<TokenBasedModal>", () => {
  it("accepted privacy policy", () => {
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
    cy.mount(TokenBasedModal);
    cy.get("#privacy-checked").parent().click();
    cy.get("#privacy-checked").should("be.checked");
    cy.contains('[data-cy="feedback-send"]').click();
    cy.contains("Betreff fehlt"); // todo fix one in a more sensible place
  });
  it("not accepted privacy policy", () => {
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
    cy.mount(TokenBasedModal);
    cy.contains("Feedback senden").click();
    cy.get('[data-cy="feedback-error"]').contains("musst die Datenschutzerklärung akzeptiert haben");
  });
  it("tokens ratelimited", () => {
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 429 });
    cy.mount(TokenBasedModal);
    cy.get('[data-cy="feedback-error"]').contains("rate-limiting");
    cy.contains("Feedback senden").should("be.disabled");
  });
  it("temporarily disabled", () => {
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 503 });
    cy.mount(TokenBasedModal);
    cy.get('[data-cy="feedback-error"]').contains("Senden von Feedback ist auf dem Server aktuell nicht konfiguriert");
    cy.contains("Feedback senden").should("be.disabled");
  });
  it("should initialise", () => {
    cy.intercept("POST", "/api/feedback/get_token", { statusCode: 201, fixture: "feedback/get_token.json" });
    cy.mount(TokenBasedModal);
    cy.get("#privacy-checked").should("not.be.checked");
    cy.contains("Feedback senden").should("not.be.disabled");
  });
});
