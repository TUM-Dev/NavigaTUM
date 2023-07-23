describe("Check if the search page works as expected", () => {
  it("searching for rooms", () => {
    cy.intercept("GET", "/api/get/root?lang=de", { fixture: "get/root.de.json" });
    cy.intercept("GET", "/api/get/5502.U1.234M?lang=de", { fixture: "get/5502.U1.234M.de.json" });
    cy.intercept("GET", "/api/search?q=f&lang=de", { fixture: "search/f.de.json" });
    cy.intercept("GET", "/api/search?q=fs&lang=de", { fixture: "search/fs.de.json" });
    cy.intercept("GET", "/api/search?q=fsm&lang=de", { fixture: "search/fsm.de.json" });
    cy.intercept("GET", "/api/search?q=fsmb&lang=de", { fixture: "search/fsmb.de.json" });
    cy.intercept("GET", "/api/search?q=fsmb&limit_buildings=10&limit_rooms=30&limit_all=30&lang=de", { fixture: "search/fsmb.long.de.json" });
    cy.visit("http://localhost:8000/");
    cy.get("input").type("fsmb");
    cy.contains("Go").click();
    cy.url().should("include", "/search?q=fsmb");
    cy.contains("5502.U1.234M");

    //use the search page again
    cy.intercept("GET", "/api/search?q=fsmp&lang=de", { fixture: "search/fsmp.de.json" });
    cy.intercept("GET", "/api/search?q=fsmpi&lang=de", { fixture: "search/fsmpi.de.json" });
    cy.intercept("GET", "/api/search?q=fsmpic&lang=de", { fixture: "search/fsmpic.de.json" });
    cy.intercept("GET", "/api/search?q=fsmpic&limit_buildings=10&limit_rooms=30&limit_all=30&lang=de", { fixture: "search/fsmpic.long.de.json" });
    cy.get("input").clear();
    cy.get("input").type("fsmpic{enter}");
    cy.url().should("include", "/search?q=fsmpic");
    cy.contains("5502.U1.234M");

    //go back
    cy.go(-1);
    cy.url().should("include", "/search?q=fsmb");
  });
  it("navigate to the details page", () => {
    cy.intercept("GET", "/api/search?q=fsmb&limit_buildings=10&limit_rooms=30&limit_all=30&lang=de", { fixture: "search/fsmb.long.de.json" });
    cy.intercept("GET", "/api/get/5502.U1.234M?lang=de", { fixture: "get/5502.U1.234M.de.json" });
    cy.visit("http://localhost:8000/search?q=fsmb");
    cy.contains("5502.U1.234M").click({ force: true });
    cy.url().should("include", "/5502.U1.234M");
  });
});

export {};
