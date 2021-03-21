
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
        let mut max_c = ParsingContext::new(&max);
        let min = i32::MIN.to_string();
        let mut min_c = ParsingContext::new(&min);

        assert_eq!(&np.parse(&mut max_c), &Ok(i32::MAX));
        assert_eq!(&np.parse(&mut min_c), &Ok(i32::MIN));

    }

    #[test]
    fn int_overflows() {
        let np = NumberParser{};
        let bigger = i32::MAX as i64 + 1;
        let bigger = bigger.to_string();
        let mut big_c = ParsingContext::new(&bigger);

        let smaller = i32::MIN as i64 - 1;
        let smaller = smaller.to_string();
        let mut small_c = ParsingContext::new(&smaller);

        assert_eq!(&np.parse(&mut big_c).is_err(), &true, "i32::MAX + 1 cannot fit in a 32-bit int");
        assert_eq!(&np.parse(&mut small_c).is_err(), &true, "i32::MIN - 1 cannot fit in a 32-bit int");
    }

    #[test]
    fn invalid_number() {
        let np = NumberParser{};
        assert_eq!(&np.parse(&mut ParsingContext::new("Asd")).is_err(), &true, "Cannot parse \"asd\" as a number");
    }

    #[test]
    fn empty_number() {
        let np = NumberParser{};
        assert_eq!(&np.parse(&mut ParsingContext::new("")).is_err(), &true, "Can't parse anything from empty string");
    }

    #[test]
    fn option_non_number() {
        let np = NumberParser{};
        let o = OptionParser::new(np);
        
        let mut c = ParsingContext::new("asd");

        assert_eq!(&o.parse(&mut c), &Ok(None), "Option parser is always successful, but might yield a None");

    }

    #[test]
    fn option_number() {
        let np = NumberParser{};
        let o = OptionParser::new(np);
        
        let mut c = ParsingContext::new("-345  ");

        assert_eq!(&o.parse(&mut c), &Ok(Some(-345)), "Option parser is always successful, but might yield a None");
    
    }
}

