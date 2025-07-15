

#[cfg(test)]
mod tests {

    use super::super::utils::*;

    #[test]
    fn test_time_conversion() {
        let secs = 3660;
        let expected = "1h 1m";
        let actual = seconds_to_human(secs);
        assert_eq!(expected, actual);
        let secs = 33000;
        let expected = "9h 10m";
        let actual = seconds_to_human(secs);
        assert_eq!(expected, actual);
        let secs = 36;
        let expected = "36s";
        let actual = seconds_to_human(secs);
        assert_eq!(expected, actual);
        let secs = 712800;
        let expected = "8d 6h";
        let actual = seconds_to_human(secs);
        assert_eq!(expected, actual);
    }
    
    #[test]
    fn test_byte_conversion() {
        let bytes = 1024;
        let expected = "1M";
        let actual = bytes_to_human(bytes);
        assert_eq!(expected, actual);
        let bytes = 1024*1024;
        let expected = "1G";
        let actual = bytes_to_human(bytes);
        assert_eq!(expected, actual);
        let bytes = 1024*100;
        let expected = "100M";
        let actual = bytes_to_human(bytes);
        assert_eq!(expected, actual);

    }

}