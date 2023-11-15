describe("Visual change", () => {
  it("main", () => {
    cy.visit("http://localhost:3000/");
    cy.contains("Standorte");
    cy.matchImage();
  });
  it("search", () => {
    cy.visit("http://localhost:3000/search?q=mw2001");
    cy.contains("Räume");
    cy.matchImage();
  });
  it("search-bar", () => {
    cy.visit("http://localhost:3000/");
    cy.get("input").type("fsmb");
    cy.wait(500);
    cy.matchImage();
  });
  it("about/privacy", () => {
    cy.visit("http://localhost:3000/about/privacy");
    cy.contains("Privacy");
    cy.matchImage();
  });
  it("about/datenschutz", () => {
    cy.visit("http://localhost:3000/about/datenschutz");
    cy.contains("Datenschutz");
    cy.matchImage();
  });
  it("about/ueber-uns", () => {
    cy.visit("http://localhost:3000/about/ueber-uns");
    cy.contains("Über Navigatum");
    cy.matchImage();
  });
  it("about/about-us", () => {
    cy.visit("http://localhost:3000/about/ueber-uns");
    cy.contains("About us");
    cy.matchImage();
  });
  it("about/impressum", () => {
    cy.visit("http://localhost:3000/about/impressum");
    cy.contains("Impressum");
    cy.matchImage();
  });
  it("about/imprint", () => {
    cy.visit("http://localhost:3000/about/imprint");
    cy.contains("Imprint");
    cy.matchImage();
  });
  it("api", () => {
    cy.visit("http://localhost:3000/api");
    cy.contains("Api");
    cy.matchImage();
  });
});
