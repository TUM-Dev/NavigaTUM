describe("Check if the search page works as expected", () => {
  it("searching for rooms", () => {
    cy.visit("http://localhost:8000/");
    cy.get("input").type("mw fachschaft");
    cy.contains("Go").click();
    cy.url().should("include", "/search?q=mw%20fachschaft");
    cy.contains("5502.U1.234M");

    //use the search page again
    cy.get("input").clear();
    cy.get("input").type("mi fachschaft{enter}");
    cy.url().should("include", "/search?q=mi%20fachschaft");
    cy.contains("5502.U1.234M");

    //go back
    cy.go(-1);
    cy.url().should("include", "/search?q=mw%20fachschaft");
  });
  it("navigate to the details page", () => {
    cy.visit("http://localhost:8000/search?q=mw%20fachschaft");
    cy.contains("5502.U1.234M").click({ force: true });
    cy.url().should("include", "/5502.U1.234M");
  });
});

export {};
