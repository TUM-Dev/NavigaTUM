describe("Check if the search page works as expected", () => {
  it("searching for rooms", () => {
    cy.intercept("GET", "/api/get/root", { statusCode: 200, fixture: "get/root.json" });
    cy.intercept("GET", "/api/get/5502.U1.234M", { statusCode: 200, fixture: "get/5502.U1.234M.json" });
    cy.intercept("GET", "/api/search?q=f", { statusCode: 200, fixture: "search?q_f.json" });
    cy.intercept("GET", "/api/search?q=fs", { statusCode: 200, fixture: "search?q_fs.json" });
    cy.intercept("GET", "/api/search?q=fsm", { statusCode: 200, fixture: "search?q_fsm.json" });
    cy.intercept("GET", "/api/search?q=fsmw", { statusCode: 200, fixture: "search?q_fsmw.json" });
    cy.visit("http://localhost:8000/");
    cy.get("input").type("fsmw");
    cy.contains("Go").click();
    cy.url().should("include", "/search?q=fsmw");
    cy.contains("5502.U1.234M");

    //use the search page again
    cy.intercept("GET", "/api/search?q=fsmp", { statusCode: 200, fixture: "search?q_fsmp.json" });
    cy.intercept("GET", "/api/search?q=fsmpi", { statusCode: 200, fixture: "search?q_fsmpi.json" });
    cy.intercept("GET", "/api/search?q=fsmpic", { statusCode: 200, fixture: "search?q_fsmpic.json" });
    cy.get("input").clear();
    cy.get("input").type("fsmpic{enter}");
    cy.url().should("include", "/search?q=fsmpic");
    cy.contains("5502.U1.234M");

    //go back
    cy.go(-1);
    cy.url().should("include", "/search?q=fsmw");
  });
  it("navigate to the details page", () => {
    cy.intercept("GET", "/api/search?q=fsmw", { statusCode: 200, fixture: "search?q_fsmw.json" });
    cy.intercept("GET", "/api/get/5502.U1.234M", { statusCode: 200, fixture: "get/5502.U1.234M.json" });
    cy.visit("http://localhost:8000/search?q=fsmw");
    cy.contains("5502.U1.234M").click({ force: true });
    cy.url().should("include", "/5502.U1.234M");
  });
});

export {};
