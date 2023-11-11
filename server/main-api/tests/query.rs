#[cfg(test)]
mod tests {
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    enum SeachFacet {
        #[serde(rename = "rooms")]
        Rooms,
        #[serde(rename = "sites_buildings")]
        SitesBuildings,
    }
    #[derive(Debug, Clone, Deserialize)]
    struct SearchEntry {
        id: String,
    }
    #[derive(Debug, Clone, Deserialize)]
    struct SearchSection {
        facet: SeachFacet,
        entries: Vec<SearchEntry>,
        n_visible: usize,
    }
    #[derive(Debug, Deserialize)]
    struct SearchRequest {
        sections: Vec<SearchSection>,
    }

    impl SearchRequest {
        fn liniarise(self) -> Vec<String> {
            let mut result = Vec::new();
            for section in self.sections.into_iter() {
                let mut copied_entries = section.entries;
                copied_entries.truncate(section.n_visible);
                for entry in copied_entries {
                    result.push(entry.id);
                }
            }
            result
        }
        async fn execute(client: &reqwest::Client, target: &str, query: &str) -> Self {
            let url = format!("{target}/api/search?q={query}");
            client
                .get(url)
                .send()
                .await
                .unwrap()
                .json::<Self>()
                .await
                .unwrap()
        }
    }

    #[derive(Default)]
    pub(crate) struct Expectation {
        target: String,
        query: String,
        among: usize,
    }

    impl Expectation {
        fn validate(&self, search_request: SearchRequest) {
            let keys = search_request.liniarise();
            for (index, id) in keys.iter().enumerate() {
                if id == &self.target {
                    assert!(
                        index < self.among,
                        "{target} (query={query}) is at {index} but expected before {among}",
                        target = self.target,
                        among = self.among,
                        query = self.query
                    );
                    return;
                }
            }
            panic!(
                "{target} not in {keys:?}[...{among}] for query {query}",
                target = self.target,
                among = self.among,
                query = self.query
            );
        }
    }

    #[derive(Default)]
    pub(crate) struct ExpectationBuilder {
        first_facet: Option<SeachFacet>,
        expectations: Vec<Expectation>,
    }

    impl ExpectationBuilder {
        fn expect_all_first_facet(self, first_facet:SeachFacet) -> Self{
            Self {
                first_facet: Some(first_facet),
                ..self
            }
        }
        fn expect_among(mut self, target: &str, query: &str, among: usize) -> Self {
            self.expectations.push(Expectation {
                target: target.to_string(),
                query: query.to_string(),
                among,
            });
            self
        }
        fn expect(mut self, target: &str, query: &str) -> Self {
            self.expectations.push(Expectation {
                target: target.to_string(),
                query: query.to_string(),
                among: 1,
            });
            self
        }
        async fn validate_against(self, target: &str) {
            let client = reqwest::Client::new();
            for exp in self.expectations {
                let result = SearchRequest::execute(&client, target, &exp.query).await;
                exp.validate(result);
            }
        }
    }

    #[tokio::test]
    async fn test_campus_search() {
        ExpectationBuilder::default()
            .expect("garching-hochbrueck", "garching-hochbrueck")
            .expect("wzw", "wzw")
            .expect_all_first_facet(SeachFacet::SitesBuildings)
            .validate_against("localhost:8080")
            .await;
    }

    #[tokio::test]
    async fn test_building_search() {
        ExpectationBuilder::default()
            .expect("5301", "5301")
            .expect("5620", "interims I")
            .expect("5416", "Interims 2")
            .expect("5304", "Mensa Garching") // Should give the 'new' mensa
            .expect("5304", "neue Mensa")
            .expect("0201", "Studitum Arcisstr") // Note: It is not really in Arcisstr., just close
            .expect("5140", "Physik Container")
            .expect("5115", "znn")
            .expect("2906", "Karlsstr. 47") // uses "str." instead of "straße"
            .expect("wzw-ziel", "ZIEL")
            .expect_all_first_facet(SeachFacet::SitesBuildings)
            .validate_against("http://localhost:8080")
            .await;
    }

    #[tokio::test]
    async fn test_exact_room_search() {
        ExpectationBuilder::default()
            .expect("5601.EG.001", "00.01.001") // A search for the Architects name should return the correct room
            .expect_all_first_facet(SeachFacet::Rooms)
            .validate_against("http://localhost:8080")
            .await;
    }

    #[tokio::test]
    async fn test_basic_room_search() {
        ExpectationBuilder::default()
            .expect("5604.EG.011", "5604.00.011") // 00 = EG
            .expect("5601.EG.001", "5601.EG.001") // MI Magistrale
            .expect_all_first_facet(SeachFacet::Rooms)
            .validate_against("http://localhost:8080")
            .await;
    }

    #[tokio::test]
    async fn test_splittable_room_search() {
        // for some rooms, splitting of strings is nessesary as a postprocessing step
        ExpectationBuilder::default()
            .expect("5508.02.801", "MW 1801")
            .expect("5508.02.801", "MW1801")
            .expect("5510.EG.001", "MW0001")
            .expect("5510.02.001", "MW2001")
            .expect_all_first_facet(SeachFacet::Rooms)
            .validate_against("http://localhost:8080")
            .await;
    }

    #[tokio::test]
    async fn test_preferences_room_search() {
        // for some rooms, splitting of strings is nessesary as a postprocessing step
        ExpectationBuilder::default()
            .expect("5101.EG.503", "2503") // lecture hall > other rooms
            .expect("5111.01.116", "1116") // seminar room > other rooms
            .expect_all_first_facet(SeachFacet::Rooms)
            .validate_against("http://localhost:8080")
            .await;
    }

        #[tokio::test]
        async fn test_room_search() {
        ExpectationBuilder::default()
            .expect("5608.03.011", "03.08.011")
            .expect("5602.EG.001", "mi hs 1")
            .expect("5508.02.801", "1801 maschinen")
            .expect("5503.EG.337", "Raum 0337 mw")
            .expect("5510.EG.001", "niemann")
            .expect("5510.EG.001", "mw g niemann")
            .expect("5402.01.220J", "CH22209")
            .expect("5101.EG.502", "pyhsik hs 2")
            .expect("5101.EG.501", "mössbauer")
            .expect("5101.EG.342", "342 Physik")
            .expect("5140.01.202", "C.3202")
            .expect("5115.01.010", "1010 znn") // Not sure about target here
            .expect("5433.EG.092", "0092@5433")
            .expect("5510.EG.026M", "0026m@5510")
            .expect("5602.EG.001", "f abuer") // typo with short word and at first letter
            .expect("5123.EG.019", "019 lmu")
            .expect("0509.EG.980", "audimax")
            .expect("0501.EG.144", "ssz") // main room called "service desk", only all 114abc.. subrooms have "ssz" in the name
            .expect("0501.EG.136", "Immathalle")
            .expect("0502.01.229", "1229 seminarraum")
            .expect("2903.02.209", "Augustenstraße 44; Raum 209; 2.OG") // Copy/paste search
            .expect_among("5604.EG.038", "praktikumsraum mi", 2) // there are two basic lab course rooms, this is one of them
            .expect_among("5604.EG.036", "physik labor mi", 3) // "5604.02.033" is a valid result before the two lab course rooms
            .expect_among("0104.U1.403", "n1403", 2) // This is "N-1403@0104", but "N1403@0104" can be before this
            .expect("5101.EG.257", "fachschaft pyhsik")
            // typo + it's "Fachschaftsbüro" in the data
            // H.003 is the correct room, but people have problems remembering how many zeroes are in the room number
            .expect("2910.EG.003", "H.0003")
            .expect("2910.EG.003", "H.003")
            .expect("2910.EG.003", "H.03")
            .expect("2910.EG.003", "H.3")
            .expect_among("0101.02.119", "2119", 4)
            // The architects name is a N2119. There are other rooms which are actually named "2119", which means 4 is the best case.
            .expect("5606.EG.011", "MI HS3")
            // This should match this Lecture hall and not the HS 1, just because both are in the Bolzmanstr. *3* 4 is the best case.
            .expect("5606.EG.011", "MI HS 3")
            .expect("0104.01.406", "N1406") // Architects names should be matchable literally
            .expect("0104.U1.406", "N-1406") // Architects names should be matchable literally
            .expect_all_first_facet(SeachFacet::Rooms)
            .validate_against("http://localhost:8080")
            .await;
    }
}
