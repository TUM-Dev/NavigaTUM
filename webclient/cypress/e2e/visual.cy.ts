describe("visual", () => {
  it("main", () => {
    cy.visit("http://localhost:3000/");
    cy.contains("Standorte");
    cy.get("#content").matchImage();
  });
  it("search", () => {
    cy.visit("http://localhost:3000/search?q=mw2001");
    cy.contains("Räume");
    cy.get("#content").matchImage();
  });
  it("search-bar", () => {
    cy.visit("http://localhost:3000/");
    cy.get("input").type("fsmb");
    cy.wait(500);
    cy.get('[data-cy="autocomplete-menu"]').matchImage();
  });
  it("about/privacy", () => {
    cy.visit("http://localhost:3000/about/privacy");
    cy.contains("Privacy Policy");
    cy.get("#content").matchImage();
  });
  it("about/datenschutz", () => {
    cy.visit("http://localhost:3000/about/datenschutz");
    cy.contains("Datenschutz");
    cy.get("#content").matchImage();
  });
  it("about/ueber-uns", () => {
    cy.visit("http://localhost:3000/about/ueber-uns");
    cy.contains("Über NavigaTUM");
    cy.get("#content").matchImage();
  });
  it("about/about-us", () => {
    cy.visit("http://localhost:3000/about/about-us");
    cy.contains("About NavigaTUM");
    cy.get("#content").matchImage();
  });
  it("about/impressum", () => {
    cy.visit("http://localhost:3000/about/impressum");
    cy.contains("Impressum");
    cy.get("#content").matchImage();
  });
  it("about/imprint", () => {
    cy.visit("http://localhost:3000/about/imprint");
    cy.contains("Imprint");
    cy.get("#content").matchImage();
  });
  it("api", () => {
    cy.visit("http://localhost:3000/api");
    cy.contains("Api");
    cy.get("#content").matchImage();
  });
});
