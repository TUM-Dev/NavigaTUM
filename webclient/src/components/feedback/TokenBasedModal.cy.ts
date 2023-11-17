import TokenBasedModal from "./TokenBasedModal.vue";

describe("<TokenBasedModal>", () => {
  it("not accepted privacy policy", () => {
    cy.intercept("POST", "/api/feedback/get_token", {statusCode: 201, fixture: "feedback/get_token.json"});
    cy.mount(TokenBasedModal);
    cy.get('[data-cy="feedback-send"]').click();
    cy.get('[data-cy="feedback-error"]').contains("musst die Datenschutzerklärung akzeptiert haben");
  });
  it("accepted privacy policy", () => {
    cy.intercept("POST", "/api/feedback/get_token", {statusCode: 201, fixture: "feedback/get_token.json"});
    cy.mount(TokenBasedModal);
    cy.get('[data-cy="feedback-privacy"]').parent().click();
    cy.get('[data-cy="feedback-privacy"]').should("be.checked");
    cy.get('[data-cy="feedback-send"]').click();
    cy.contains("Betreff fehlt"); // todo fix one in a more sensible place
  });
  it("tokens ratelimited", () => {
    cy.intercept("POST", "/api/feedback/get_token", {statusCode: 429});
    cy.mount(TokenBasedModal);
    cy.get('[data-cy="feedback-error"]').contains("rate-limiting");
    cy.get('[data-cy="feedback-send"]').should("be.disabled");
  });
  it("temporarily disabled", () => {
    cy.intercept("POST", "/api/feedback/get_token", {statusCode: 503});
    cy.mount(TokenBasedModal);
    cy.get('[data-cy="feedback-error"]').contains("Senden von Feedback ist auf dem Server aktuell nicht konfiguriert");
    cy.get('[data-cy="feedback-send"]').should("be.disabled");
  });
  it('should initialise', () => {
    cy.intercept("POST", "/api/feedback/get_token", {statusCode: 201, fixture: "feedback/get_token.json"});
    cy.mount(TokenBasedModal);
    cy.get('[data-cy="feedback-privacy"]').should("not.be.checked");
    cy.get('[data-cy="feedback-send"]').should("not.be.disabled");
  });
});
