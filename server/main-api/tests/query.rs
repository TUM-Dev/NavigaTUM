

    use rstest::rstest;
    use pretty_assertions::assert_eq;

    #[rstest]  
    // site/area/campus queries
    #[case("garching-hochbrueck", "hochbrück", None)]
    #[case("wzw", "wzw", None)]
    // building queries
    #[case("5301", "5301", None)]
    #[case("5620", "interims I", None)]
    #[case("5416", "Interims 2", None)]
    #[case("5304", "Mensa Garching", None)] // Should give the 'new' mensa
    #[case("5304", "neue Mensa", None)]
    #[case("0201", "Studitum Arcisstr", None)] // Note: It is not really in Arcisstr., just close
    #[case("5140", "Physik Container", None)]
    #[case("5115", "znn", None)]
    #[case("2906", "Karlsstr. 47", None)] // uses "str." instead of "straße"
    #[case("wzw-ziel", "ZIEL", None)]
    // room queries
    #[case("5604.EG.011", "5604.00.011", None)]
    #[case("5601.EG.001", "5601.EG.001", None)] // MI Magistrale
    #[case("5601.EG.001", "00.01.001", None)] // A search for the Architects name should return the correct room
    #[case("5608.03.011", "03.08.011", None)]
    #[case("5602.EG.001", "mi hs 1", None)]
    #[case("5508.02.801", "MW 1801", None)]
    #[case("5508.02.801", "MW1801", None)] // splitting necessary
    #[case("5510.EG.001", "MW0001", None)] // splitting nessesary
    #[case("5510.02.001", "MW2001", None)] // splitting nessesary
    #[case("5508.02.801", "1801 maschinen", None)]
    #[case("5503.EG.337", "Raum 0337 mw", None)]
    #[case("5510.EG.001", "niemann", None)]
    #[case("5510.EG.001", "mw g niemann", None)]
    #[case("5402.01.220J", "CH22209", None)]
    #[case("5101.EG.502", "pyhsik hs 2", None)]
    #[case("5101.EG.501", "mössbauer", None)]
    #[case("5101.EG.342", "342 Physik", None)]
    #[case("5101.EG.503", "2503", None)] // lecture hall, should be preferred over other rooms
    #[case("5111.01.116", "1116", None)] // seminar room, should be preferred over other rooms
    #[case("5140.01.202", "C.3202", None)]
    #[case("5115.01.010", "1010 znn", None)] // Not sure about target here
    #[case("5433.EG.092", "0092@5433", None)]
    #[case("5510.EG.026M", "0026m@5510", None)]
    #[case("5602.EG.001", "f abuer", None)] // typo with short word and at first letter
    #[case("5123.EG.019", "019 lmu", None)]
    #[case("0509.EG.980", "audimax", None)]
    #[case("0501.EG.144", "ssz", None)] // main room called "service desk", only all 114abc.. subrooms have "ssz" in the name
    #[case("0501.EG.136", "Immathalle", None)]
    #[case("0502.01.229", "1229 seminarraum", None)]
    #[case("2903.02.209", "Augustenstraße 44; Raum 209; 2.OG", None)] // Copy/paste search
    #[case("5604.EG.038", "praktikumsraum mi", Some(2))] // there are two basic lab course rooms, this is one of them
    #[case("5604.EG.036", "physik labor mi", Some(3))] // "5604.02.033" is a valid result before the two lab course rooms
    #[case("0104.U1.403", "n1403", Some(2))] // This is "N-1403@0104", but "N1403@0104" can be before this
    #[case("5101.EG.257", "fachschaft pyhsik", None)] // typo + it's "Fachschaftsbüro" in the data
    // H.003 is the correct room, but people have problems remembering how many zeroes are in the room number
    #[case("2910.EG.003", "H.0003", None)]
    #[case("2910.EG.003", "H.003", None)]
    #[case("2910.EG.003", "H.03", None)]
    #[case("2910.EG.003", "H.3", None)]
    #[case("0101.02.119", "2119", Some(4))] // The architects name is a N2119. There are other rooms which are actually named "2119", which means 4 is the best case.
    #[case("5606.EG.011", "MI HS3", None)] // This should match this Lecture hall and not the HS 1, just because both are in the Bolzmanstr. *3* 4 is the best case.
    #[case("5606.EG.011", "MI HS 3", None)]
    #[case("0104.01.406", "N1406", None)] // Architects names should be matachable literally
    #[case("0104.U1.406", "N-1406", None)] // Architects names should be matachable literally
    // other queries (currently unsupported)
    //#- {target: , "mathe bib", None)]
    //#- {target: , "tb innenstadt", None)]
    pub fn search_test(#[case] target: &str,#[case] query: &str,#[case] among: Option<usize>) {
        let search_endpoint = "https://nav.tum.de/api/search";
        assert_eq!(target,query)
    }

    #[rstest]
    #[case(0, 0)]
    #[case(1, 1)]
    #[case(2, 1)]
    #[case(3, 2)]
    #[case(4, 3)]
    fn fibonacci_test(#[case] input: u32, #[case] expected: u32) {
        assert_eq!(expected, fibonacci(input))
    }

fn fibonacci(input: u32)->u32{
    input
}