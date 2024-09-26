#[derive(Debug, PartialEq)]
pub enum UnpackError {
    InvalidInput(String),
}

pub fn unpack_string(input: &str) -> Result<String, UnpackError> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        println!("{}", c.is_digit(10));
        if c == '\\' {
            if let Some(next_char) = chars.next() {
                result.push(next_char);
            } else {
                return Err(UnpackError::InvalidInput("Trailing backslash".to_string()));
            }
        } else if c.is_digit(10) {
            // Обработка некорректных строк, начинающихся с цифры
            if result.is_empty() {
                return Err(UnpackError::InvalidInput("Leading digit".to_string()));
            }
            let mut count = c.to_digit(10).unwrap() as usize;

            // Обработка возможных многозначных чисел
            while let Some(&next_c) = chars.peek() {
                if next_c.is_digit(10) {
                    chars.next();
                    count = count * 10 + next_c.to_digit(10).unwrap() as usize;
                } else {
                    break;
                }
            }

            if let Some(last_char) = result.chars().last() {
                result.push_str(&last_char.to_string().repeat(count - 1));
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn main() {
    let result = unpack_string("qwe\\4\\5");
    println!("{}", result.unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(unpack_string("a4bc2d5e").unwrap(), "aaaabccddddde");
    }

    #[test]
    fn test2() {
        assert_eq!(unpack_string("abcd").unwrap(), "abcd");
    }

    #[test]
    fn test3() {
        assert_eq!(unpack_string("").unwrap(), "");
    }

    #[test]
    fn test4() {
        assert_eq!(unpack_string("45").unwrap_err(), UnpackError::InvalidInput("Leading digit".to_string()));
    }

    #[test]
    fn test5() {
        assert_eq!(unpack_string("qwe\\4\\5").unwrap(), "qwe45");
    }

    #[test]
    fn test6() {
        assert_eq!(unpack_string("qwe\\45").unwrap(), "qwe44444");
    }

    #[test]
    fn test7() {
        assert_eq!(unpack_string("qwe\\\\5").unwrap(), "qwe\\\\\\\\\\");
    }

    #[test]
    fn test8() {
        assert_eq!(unpack_string("a\\bc\\d").unwrap(), "abcd");
    }

    #[test]
    fn test9() {
        assert_eq!(unpack_string("qwe\\").unwrap_err(), UnpackError::InvalidInput("Trailing backslash".to_string()));
    }
}

