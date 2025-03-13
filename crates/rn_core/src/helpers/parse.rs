use std::str::FromStr;
use std::fmt::Debug;

use super::errors::{ParseError, ParseResult};

/// Trait for parsing operands from string to numeric types
/// Handles both hexadecimal (with $ prefix) and decimal formats
pub trait ParseOperand: Sized {
    /// Parse a string into a numeric type, handling both hex and decimal formats
    /// Returns the parsed value or an error if parsing fails
    fn parse_operand(input: &str) -> ParseResult<Self>;

    /// Get the name of the type for error messages
    fn type_name() -> &'static str;
}

/// Trait for types that can be parsed from a string with a given radix
pub trait FromStrRadix: Sized {
    /// Parse a string with the given radix
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, std::num::ParseIntError>;
}

// Implement FromStrRadix for u8 and u16
impl FromStrRadix for u8 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        u8::from_str_radix(s, radix)
    }
}

impl FromStrRadix for u16 {
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        u16::from_str_radix(s, radix)
    }
}

/// Helper function for parsing a numeric value from a string, handling hex and decimal formats
fn parse_numeric_value<T>(input: &str, type_name: &str) -> ParseResult<T> 
where 
    T: FromStr + FromStrRadix + Debug,
    <T as std::str::FromStr>::Err: std::fmt::Debug
{
    // First remove '#' prefix if it exists (for immediate addressing mode)
    let input = if input.starts_with('#') {
        &input[1..]
    } else {
        input
    };
    
    let input = input.trim();
    
    // Handle hexadecimal format (with $ prefix)
    if input.starts_with('$') {
        let hex_str = &input[1..];
        T::from_str_radix(hex_str, 16)
            .map_err(|_| ParseError::InvalidFormat(
                format!("Invalid hex {} value: ${}", type_name, hex_str)
            ))
    } 
    // Handle binary format (with % prefix)
    else if input.starts_with('%') {
        let bin_str = &input[1..];
        T::from_str_radix(bin_str, 2)
            .map_err(|_| ParseError::InvalidFormat(
                format!("Invalid binary {} value: %{}", type_name, bin_str)
            ))
    }
    // Handle decimal format
    else {
        input.parse::<T>()
            .map_err(|_| ParseError::InvalidFormat(
                format!("Invalid decimal {} value: {}", type_name, input)
            ))
    }
}

impl ParseOperand for u8 {
    fn parse_operand(input: &str) -> ParseResult<Self> {
        parse_numeric_value(input, Self::type_name())
    }

    fn type_name() -> &'static str {
        "byte"
    }
}

impl ParseOperand for u16 {
    fn parse_operand(input: &str) -> ParseResult<Self> {
        parse_numeric_value(input, Self::type_name())
    }

    fn type_name() -> &'static str {
        "word"
    }
}

/// Helper function to parse a value with optional $ prefix (hex) or as decimal
/// Also handles immediate addressing mode with # prefix
/// Generic over any type that implements ParseOperand
pub fn parse_value<T: ParseOperand>(input: &str) -> ParseResult<T> {
    T::parse_operand(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u8_hex() {
        // Valid hex values
        assert_eq!(parse_value::<u8>("$00").unwrap(), 0);
        assert_eq!(parse_value::<u8>("$FF").unwrap(), 255);
        assert_eq!(parse_value::<u8>("$0F").unwrap(), 15);

        // Invalid hex values
        assert!(parse_value::<u8>("$100").is_err()); // Out of range
        assert!(parse_value::<u8>("$ZZ").is_err());  // Invalid characters
    }

    #[test]
    fn test_parse_u8_binary() {
        // Valid binary values
        assert_eq!(parse_value::<u8>("%00000000").unwrap(), 0);
        assert_eq!(parse_value::<u8>("%11111111").unwrap(), 255);
        assert_eq!(parse_value::<u8>("%00001111").unwrap(), 15);
        assert_eq!(parse_value::<u8>("%00010000").unwrap(), 16);

        // Invalid binary values
        assert!(parse_value::<u8>("%111111111").is_err()); // Too many bits for u8
        assert!(parse_value::<u8>("%1234").is_err());      // Invalid binary digits
    }

    #[test]
    fn test_parse_u8_decimal() {
        // Valid decimal values
        assert_eq!(parse_value::<u8>("0").unwrap(), 0);
        assert_eq!(parse_value::<u8>("255").unwrap(), 255);
        assert_eq!(parse_value::<u8>("15").unwrap(), 15);

        // Invalid decimal values
        assert!(parse_value::<u8>("256").is_err()); // Out of range
        assert!(parse_value::<u8>("-1").is_err());  // Negative value
        assert!(parse_value::<u8>("12.34").is_err()); // Float value
    }

    #[test]
    fn test_parse_u16_hex() {
        // Valid hex values
        assert_eq!(parse_value::<u16>("$0000").unwrap(), 0);
        assert_eq!(parse_value::<u16>("$FFFF").unwrap(), 65535);
        assert_eq!(parse_value::<u16>("$1234").unwrap(), 0x1234);

        // Invalid hex values
        assert!(parse_value::<u16>("$10000").is_err()); // Out of range
        assert!(parse_value::<u16>("$WXYZ").is_err());  // Invalid characters
    }

    #[test]
    fn test_parse_u16_decimal() {
        // Valid decimal values
        assert_eq!(parse_value::<u16>("0").unwrap(), 0);
        assert_eq!(parse_value::<u16>("65535").unwrap(), 65535);
        assert_eq!(parse_value::<u16>("1234").unwrap(), 1234);

        // Invalid decimal values
        assert!(parse_value::<u16>("65536").is_err()); // Out of range
        assert!(parse_value::<u16>("-1").is_err());    // Negative value
    }

    #[test]
    fn test_immediate_operand() {
        // Test with # prefix for hex values
        assert_eq!(parse_value::<u8>("#$42").unwrap(), 0x42);
        assert_eq!(parse_value::<u16>("#$1234").unwrap(), 0x1234);
        
        // Test with # prefix for binary values
        assert_eq!(parse_value::<u8>("#%00101010").unwrap(), 42);  // Binary 00101010 = decimal 42
        assert_eq!(parse_value::<u8>("#%00010000").unwrap(), 16);  // Binary 00010000 = decimal 16
        
        // Test without # prefix (should still work)
        assert_eq!(parse_value::<u8>("$42").unwrap(), 0x42);
        assert_eq!(parse_value::<u16>("$1234").unwrap(), 0x1234);
        assert_eq!(parse_value::<u8>("%00101010").unwrap(), 42);
    }
} 