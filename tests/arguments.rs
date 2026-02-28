use stannum::arguments;

#[test]
fn empty_line() {
    let value = "";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Ok(vec![]));
}

#[test]
fn comma_seperated_lines() {
    let value = "1,2,3,5,7,10";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Ok(vec![1, 2, 3, 5, 7, 10]));
}

#[test]
fn range_of_lines() {
    let value = "5-15";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Ok(vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]));
}

#[test]
fn range_of_lines_unsorted_duplicates() {
    let value = "7,1,7,9,2,12,6,3";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Ok(vec![1, 2, 3, 6, 7, 9, 12]));
}

#[test]
fn range_comma_lines() {
    let value = "2-10,8-15";
    let result = arguments::parse_lines(value);
    assert_eq!(
        result,
        Ok(vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
    );
}

#[test]
fn malformed_lines() {
    let value = "-";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Err("Invalid integer!".to_string()));

    // Should not be possible in the context of command line arguments, as this would be
    // interpreted as its own flag but whatever
    let value = "-2";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Err("Invalid integer!".to_string()));

    let value = "2-";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Err("Invalid integer!".to_string()));
}

#[test]
fn invalid_line() {
    let value = "10,5,7,0";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Err("Invalid line: 0!".to_string()));

    let value = "0-10";
    let result = arguments::parse_lines(value);
    assert_eq!(result, Err("Invalid line: 0!".to_string()));
}

#[test]
fn empyt_columns() {
    let value = "";
    let result = arguments::parse_line_columns(value);
    assert_eq!(result, Ok(vec![]));
}

#[test]
fn single_line_column() {
    let value = "1,2,20";
    let result = arguments::parse_line_columns(value);
    assert_eq!(result, Ok(vec![(1, 2, 20)]));
}

#[test]
fn multiple_line_column() {
    let value = "1,2,20;1,4,15;5,6,6";
    let result = arguments::parse_line_columns(value);
    assert_eq!(result, Ok(vec![(1, 2, 20), (1, 4, 15), (5, 6, 6)]));
}

#[test]
fn multiple_line_column_unsorted() {
    let value = "5,10,15;6,2,2;1,3,13";
    let result = arguments::parse_line_columns(value);
    assert_eq!(result, Ok(vec![(1, 3, 13), (5, 10, 15), (6, 2, 2)]));
}

#[test]
fn malformed_column() {
    let value = "1,20,2";
    let result = arguments::parse_line_columns(value);
    assert_eq!(result, Err("Invalid column range!".to_string()));
}
