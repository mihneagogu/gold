
#[cfg(test)]
mod parsing_tests {

    use crate::*;

    #[test]
    fn int_max_and_min() {
        let max = i32::MAX.to_string();
        let mut max_c = ParsingContext::new(&max);
        let min = i32::MIN.to_string();
        let mut min_c = ParsingContext::new(&min);

        assert_eq!(parse_number(&mut max_c), Ok(i32::MAX));
        assert_eq!(parse_number(&mut min_c), Ok(i32::MIN));

    }

    #[test]
    fn int_overflows() {
        let bigger = i32::MAX as i64 + 1;
        let bigger = bigger.to_string();
        let mut big_c = ParsingContext::new(&bigger);

        let smaller = i32::MIN as i64 - 1;
        let smaller = smaller.to_string();
        let mut small_c = ParsingContext::new(&smaller);

        assert_eq!(parse_number(&mut big_c).is_err(), true, "i32::MAX + 1 cannot fit in a 32-bit int");
        assert_eq!(parse_number(&mut small_c).is_err(), true, "i32::MIN - 1 cannot fit in a 32-bit int");
    }

    #[test]
    fn invalid_number() {
        assert_eq!(parse_number(&mut ParsingContext::new("Asd")).is_err(), true, "Cannot parse \"asd\" as a number");
    }

    #[test]
    fn empty_number() {
        assert_eq!(parse_number(&mut ParsingContext::new("")).is_err(), true, "Can't parse anything from empty string");

    }
}

