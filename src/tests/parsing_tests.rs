
#[cfg(test)]
mod parsing_tests {

    use crate::*;

    #[test]
    fn int_max_and_min() {
        let max = i32::MAX.to_string();
        let min = i32::MIN.to_string();
        assert_eq!(parse_number(max.as_str()), Ok(i32::MAX));
        assert_eq!(parse_number(min.as_str()), Ok(i32::MIN));

    }

    #[test]
    fn int_overflows() {
        let bigger = i32::MAX as i64 + 1;
        let bigger = bigger.to_string();
        let smaller = i32::MIN as i64 - 1;
        let smaller = smaller.to_string();

        assert_eq!(parse_number(bigger.as_str()).is_err(), true);
        assert_eq!(parse_number(smaller.as_str()).is_err(), true);

    }
}

