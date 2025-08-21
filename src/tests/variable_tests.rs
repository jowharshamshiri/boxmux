#[cfg(test)]
mod variable_tests {
    use super::*;
    use crate::model::app::substitute_variables;
    use std::env;

    #[test]
    fn test_basic_variable_substitution() {
        env::set_var("TEST_VAR", "test_value");
        
        let input = "Hello ${TEST_VAR}!";
        let result = substitute_variables(input).unwrap();
        
        assert_eq!(result, "Hello test_value!");
        
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_variable_with_default() {
        env::remove_var("MISSING_VAR");
        
        let input = "Value: ${MISSING_VAR:default_value}";
        let result = substitute_variables(input).unwrap();
        
        assert_eq!(result, "Value: default_value");
    }

    #[test] 
    fn test_simple_env_var() {
        env::set_var("SIMPLE_VAR", "simple_value");
        
        let input = "Simple: $SIMPLE_VAR";
        let result = substitute_variables(input).unwrap();
        
        assert_eq!(result, "Simple: simple_value");
        
        env::remove_var("SIMPLE_VAR");
    }

    #[test]
    fn test_mixed_variables() {
        env::set_var("APP_NAME", "BoxMux");
        env::set_var("USER", "developer");
        
        let input = "${APP_NAME} user: ${USER} home: ${HOME:/tmp}";
        let result = substitute_variables(input).unwrap();
        
        assert!(result.contains("BoxMux user: developer"));
        
        env::remove_var("APP_NAME");
        env::remove_var("USER");
    }

    #[test]
    fn test_no_variables() {
        let input = "No variables here";
        let result = substitute_variables(input).unwrap();
        
        assert_eq!(result, "No variables here");
    }

    #[test]
    fn test_empty_default() {
        env::remove_var("EMPTY_VAR");
        
        let input = "Empty: ${EMPTY_VAR:}";
        let result = substitute_variables(input).unwrap();
        
        assert_eq!(result, "Empty: ");
    }
}