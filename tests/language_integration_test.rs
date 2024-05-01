

#[test]
fn test_add() {
    let result = hydrogen::main("test_files/if_condition_basic.bk".to_owned());
    assert!(result.is_ok(), "Result is error");
    let result = result.unwrap();
    assert_eq!(result, "1");
}
