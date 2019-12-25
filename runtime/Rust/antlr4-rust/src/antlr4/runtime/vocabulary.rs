use std::cmp;

use crate::antlr4::runtime::token::TokenType;

pub trait Vocabulary {
    fn get_max_token_type() -> i32;
    fn get_literal_name(token_type: i32) -> String;
    fn get_symbolic_name(token_type: i32) -> String;
}

pub struct VocabularyImpl {
    literal_names: Vec<Option<String>>,
    symbolic_names: Vec<Option<String>>,
    display_names: Vec<Option<String>>,
    max_token_type: i32,
}

impl VocabularyImpl {
    pub fn new(
        literal_names: Vec<Option<String>>,
        symbolic_names: Vec<Option<String>>,
        display_names: Vec<Option<String>>,
    ) -> VocabularyImpl {
        let mtt = cmp::max(display_names.len(), cmp::max(literal_names.len(), symbolic_names.len())) - 1;
        VocabularyImpl {
            literal_names: literal_names,
            symbolic_names: symbolic_names,
            display_names: display_names,
            max_token_type: mtt as i32,
        }
    }
    pub fn from_token_names(token_names:Vec<Option<String>>) -> VocabularyImpl {
        if token_names.len() == 0 {
            return VocabularyImpl::new(Vec::new(), Vec::new(), Vec::new());
        } else {
            let mut literal_names:Vec<Option<String>> = token_names.clone();
            let mut symbolic_names:Vec<Option<String>> = token_names.clone();
            
            for i in 0..token_names.len() {
                if let Some(token_name) = &token_names[i] {
                    let first_char = token_name.chars().next().unwrap();
                    if first_char == '\'' {
                        symbolic_names[i] = None;
                        continue
                    } else if first_char.is_uppercase() {
                        literal_names[i] = None;
                        continue
                    }
                }
                literal_names[i] = None;
                symbolic_names[i] = None;
            }
            return VocabularyImpl::new(literal_names, symbolic_names, token_names);
        }
    }

    pub fn get_max_token_type(&self) -> i32 {
        return self.max_token_type;
    }

    pub fn get_literal_name(&self, token_type:i32) -> Option<String> {
        if token_type >= 0 && token_type < (self.literal_names.len() as i32) {
            return self.literal_names[token_type as usize].clone();
        }
        return None;
    }

    pub fn get_symbolic_name(&self, token_type: i32) -> Option<String> {
        if token_type >= 0 && token_type < (self.symbolic_names.len() as i32) {
            return self.symbolic_names[token_type as usize].clone();
        }
        if token_type == TokenType::EOF.value() {
            return Some(String::from("EOF"));
        }
        return None;
    }

    pub fn get_display_name(&self, token_type:i32) -> Option<String> {
        if token_type >= 0 && token_type < (self.display_names.len() as i32) {
            return self.display_names[token_type as usize].clone();
        }

        if let Some(ln) = self.get_literal_name(token_type) {
            return Some(ln);
        }

        if let Some(sn) = self.get_symbolic_name(token_type) {
            return Some(sn);
        }
        return None;
    }
}
