
#[cfg(test)]
mod parsing_tests {

    use parsing::literals::NumberParser;
    use parsing::combinators::OptionParser;

    use crate::*;
    use crate::parsing::Parser;

    #[test]
    fn int_max_and_min() {
        let np = NumberParser{};
        let max = i32::MAX.to_string();
        let min = i32::MIN.to_string();

        assert_eq!(&np.run_parser(&max), &Ok(i32::MAX));
        assert_eq!(&np.run_parser(&min), &Ok(i32::MIN));

    }

    #[test]
    fn int_overflows() {
        let np = NumberParser{};
        let bigger = i32::MAX as i64 + 1;
        let bigger = bigger.to_string();

        let smaller = i32::MIN as i64 - 1;
        let smaller = smaller.to_string();

        assert_eq!(&np.run_parser(&bigger).is_err(), &true, "i32::MAX + 1 cannot fit in a 32-bit int");
        assert_eq!(&np.run_parser(&smaller).is_err(), &true, "i32::MIN - 1 cannot fit in a 32-bit int");
    }

    #[test]
    fn invalid_number() {
        let np = NumberParser{};
        assert_eq!(&np.run_parser("asd").is_err(), &true, "Cannot parse \"asd\" as a number");
    }

    #[test]
    fn empty_number() {
        let np = NumberParser{};
        assert_eq!(&np.run_parser("").is_err(), &true, "Can't parse anything from empty string");
    }

    #[test]
    fn option_non_number() {
        let np = NumberParser{};
        let o = OptionParser::new(np);
        

        assert_eq!(&o.run_parser("asd"), &Ok(None), "Option parser is always successful, but might yield a None");

    }

    #[test]
    fn option_number() {
        let np = NumberParser{};
        let o = OptionParser::new(np);
        
        assert_eq!(&o.run_parser("-345  "), &Ok(Some(-345)), "Option parser is always successful, but might yield a None");
    }

    #[test]
    fn alternative_parser_test() {
        // Tests whether the alternative correctly parses its variants
        // Use StringParser with attempt for this one since we know it will never parse asd first
        let asd = StringParser::new("asd");
        // This should succeed since we didn't eat any input with the first parser
        let one = StringParser::new("123");

        let ap = vec![&asd as &dyn Parser<Output = &'static str, PErr = StringParseErr>, 
        &one as &dyn Parser<Output = &'static str, PErr = StringParseErr>];
        let ap = AlternativeParser::new(ap);

        assert_eq!(ap.run_parser(" \n 123 as").is_ok(), true);
    }

    #[test]
    fn alternative_parser_unsuccessful_test() {
        // Tests whether the alternative correctly parses its variants
        // This will eat part of the input, which means the parser ONE won't match anymore, since
        // we ate the "123" in the string with the ASD parser
        let asd = RawStringParser::new("asd");
        // This won't succeed since we ate "123"
        let one = RawStringParser::new("123");

        let ap = vec![&asd as &dyn Parser<Output = &'static str, PErr = StringParseErr>, 
        &one as &dyn Parser<Output = &'static str, PErr = StringParseErr>];
        let ap = AlternativeParser::new(ap);

        assert_eq!(ap.run_parser(" \n 123 as").is_err(), true);
    }

    #[test]
    fn parsing_keywords_works() {
        let sp = StringParser::new("def");

        // This one should succeed because it is parsing "def", which is a keyword
        assert_eq!(sp.run_parser(" \n def").is_ok(), true);
        // This one should fail because we are trying to parse the "def" keyword,
        // but we actually get the "def_" identifier
        assert_eq!(sp.run_parser(" \n def_").is_err(), true);

    }

    #[test]
    fn ident_tests() {
        let ip = IdentParser{};
        // Need at least one alpha-num char
        assert_eq!(ip.run_parser("____").is_err(), true);
        assert_eq!(ip.run_parser("asd").is_ok(), true);
        // Can't start with a digit
        assert_eq!(ip.run_parser("3asd").is_err(), true);
        // Can't define keywords as identifiers
        assert_eq!(ip.run_parser("def").is_err(), true);
        // This is fine
        assert_eq!(ip.run_parser("_def").is_ok(), true);
        // This is also fine but PLEASE do not name your variables like that
        assert_eq!(ip.run_parser("__def__").is_ok(), true);
        let mut s = String::from("__def__");
        let c: char = char::from_u32(0xd8342).unwrap();
        s.push(c);
        // Can't have identifiers with unicode in them
        assert_eq!(ip.run_parser(&s).is_err(), true);

    }

    #[test]
    fn simple_type_tests() {
        let t = Type();
        assert_eq!(t.run_parser("i32").is_ok(), true);
        assert_eq!(t.run_parser("  SomeStruct ").is_ok(), true);
        assert_eq!(t.run_parser("  ").is_err(), true);
    }

    #[test]
    fn ptr_and_ref_type_tests() {
        let t = Type();
        assert_eq!(t.run_parser("&i32").is_ok(), true);
        assert_eq!(t.run_parser("  **SomeStruct ").is_ok(), true);
        assert_eq!(t.run_parser("  & & * SomeStruct").is_ok(), true);
    }

    #[test]
    fn generic_types_tests() {
        let t = Type();
        assert_eq!(t.run_parser("Vec<i32>").is_ok(), true);
        assert_eq!(t.run_parser("  Hashmap<Asd, Vec<Mike, El>> ").is_ok(), true);
        assert_eq!(t.run_parser("  Pair<&i32, *&Mike>").is_ok(), true);
    }

    
}

