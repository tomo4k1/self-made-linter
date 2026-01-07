use html5gum::{Emitter, Error, State};
use oxc_span::Span;
use std::collections::BTreeMap;
use std::str;

#[derive(Debug, Clone)]
pub enum TemplateToken {
    StartTag {
        name: String,
        attributes: BTreeMap<String, String>,
        self_closing: bool,
        span: Span,
    },
    EndTag {
        name: String,
        span: Span,
    },
    String {
        content: String,
        span: Span,
    },
    #[allow(dead_code)]
    Comment {
        content: String,
        span: Span,
    },
    Eof,
}

pub struct SpannedEmitter {
    base_ptr: usize,
    base_len: usize,
    
    // Internal state for constructing tokens
    current_token_start: usize,
    current_token_end: usize,
    
    current_is_end_tag: bool, // Track if current tag being built is end tag
    current_tag_name: String,
    current_attributes: BTreeMap<String, String>,
    
    current_attr_name: String,
    current_attr_value: String,
    
    self_closing: bool,
    
    emitted_tokens: Rc<RefCell<VecDeque<TemplateToken>>>,
    #[allow(dead_code)]
    errors: Vec<Error>,
}

use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;

impl SpannedEmitter {
    pub fn new(input: &str) -> (Self, Rc<RefCell<VecDeque<TemplateToken>>>) {
        let tokens = Rc::new(RefCell::new(VecDeque::new()));
        let emitter = Self {
            base_ptr: input.as_ptr() as usize,
            base_len: input.len(),
            current_token_start: usize::MAX,
            current_token_end: 0,
            current_is_end_tag: false,
            current_tag_name: String::new(),
            current_attributes: BTreeMap::new(),
            current_attr_name: String::new(),
            current_attr_value: String::new(),
            self_closing: false,
            emitted_tokens: tokens.clone(),
            errors: Vec::new(),
        };
        (emitter, tokens)
    }

    #[allow(dead_code)]
    pub fn finish(self) -> Vec<TemplateToken> {
        // This consumes the helper, but the Rc is shared.
        // We can just return what's in the Rc.
        let mut tokens = self.emitted_tokens.borrow_mut();
        tokens.drain(..).collect()
    }

    fn update_span(&mut self, s: &[u8]) {
        let ptr = s.as_ptr() as usize;
        // Simple bounds check to ensure we are looking at original input slice
        if ptr >= self.base_ptr && ptr < self.base_ptr + self.base_len {
            let start = ptr - self.base_ptr;
            let end = start + s.len();
            if start < self.current_token_start {
                self.current_token_start = start;
            }
            if end > self.current_token_end {
                self.current_token_end = end;
            }
        }
    }
    
    fn reset_current(&mut self) {
        self.current_token_start = usize::MAX;
        self.current_token_end = 0;
        self.current_is_end_tag = false;
        self.current_tag_name.clear();
        self.current_attributes.clear();
        self.current_attr_name.clear();
        self.current_attr_value.clear();
        self.self_closing = false;
    }
    
    fn create_span(&self, is_tag: bool) -> Span {
        let start = if self.current_token_start == usize::MAX { 0 } else { self.current_token_start };
        let mut end = self.current_token_end;
        if start > end { end = start; }
        
        // For tags, start is usually adjusted to include '<' because html5gum might start token at name?
        // Actually html5gum `push_tag_name` updates span.
        // `update_span` uses pointer arithmetic.
        // If I use `html5gum`, strictly speaking `init_start_tag` might not give me position.
        // But `html5gum` consumes `<`.
        
        let effective_start = if is_tag && start > 0 { start - 1 } else { start };
        Span::new(effective_start as u32, end as u32)
    }
}

impl Emitter for SpannedEmitter {
    type Token = TemplateToken;

    fn emit_string(&mut self, s: &[u8]) {
        self.update_span(s);
        let content = String::from_utf8_lossy(s).to_string();
        self.emitted_tokens.borrow_mut().push_back(TemplateToken::String {
            content,
            span: self.create_span(false),
        });
        self.reset_current();
    }

    fn emit_eof(&mut self) {
        self.emitted_tokens.borrow_mut().push_back(TemplateToken::Eof);
    }

    fn emit_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn pop_token(&mut self) -> Option<Self::Token> {
        self.emitted_tokens.borrow_mut().pop_front()
    }

    fn set_last_start_tag(&mut self, _: Option<&[u8]>) {}
    
    fn init_start_tag(&mut self) { 
        self.reset_current(); 
        self.current_is_end_tag = false;
    }
    
    fn init_end_tag(&mut self) { 
        self.reset_current(); 
        self.current_is_end_tag = true;
    }
    
    fn init_comment(&mut self) { 
        self.reset_current(); 
    }
    
    fn emit_current_tag(&mut self) -> Option<State> {
        // Flush any pending attribute
        if !self.current_attr_name.is_empty() {
            self.current_attributes.insert(self.current_attr_name.clone(), self.current_attr_value.clone());
            self.current_attr_name.clear();
            self.current_attr_value.clear();
        }

        let span = self.create_span(true);
        let name = self.current_tag_name.clone();
        
        if self.current_is_end_tag {
            self.emitted_tokens.borrow_mut().push_back(TemplateToken::EndTag {
                name,
                span,
            });
        } else {
            self.emitted_tokens.borrow_mut().push_back(TemplateToken::StartTag {
                name,
                attributes: self.current_attributes.clone(),
                self_closing: self.self_closing,
                span,
            });
        }
        
        self.reset_current();
        None
    }
    
    fn emit_current_comment(&mut self) {
        // For now, ignore comments in terms of token output or simple placeholder
        let span = self.create_span(false); // Comments start with <!--, treated as data? Or tag?
        // html5gum `emit_current_comment`. 
        // `update_span` sees data inside comment?
        // Let's assume false for now, or true if I want to capture `<!--`.
        // html5gum handles `<!--` consumption.
        // My heuristic `start-1` implies `match_start` points to char AFTER `<`.
        // For comment, it might be complicated.
        // But comments are not used by rules yet.
        self.emitted_tokens.borrow_mut().push_back(TemplateToken::Comment {
            content: self.current_tag_name.clone(), 
            span
        });
        self.reset_current();
    }
    
    fn emit_current_doctype(&mut self) {
        self.reset_current();
    }
    
    fn set_self_closing(&mut self) { self.self_closing = true; }
    fn set_force_quirks(&mut self) {}
    
    fn push_tag_name(&mut self, s: &[u8]) { 
        self.update_span(s);
        self.current_tag_name.push_str(&String::from_utf8_lossy(s));
    }
    
    fn push_comment(&mut self, s: &[u8]) { 
        self.update_span(s); 
        // Reuse current_tag_name buffer for comment content for simplicity
        self.current_tag_name.push_str(&String::from_utf8_lossy(s));
    }
    
    fn push_doctype_name(&mut self, _s: &[u8]) {}
    fn init_doctype(&mut self) {}
    
    fn init_attribute(&mut self) {
        if !self.current_attr_name.is_empty() {
            self.current_attributes.insert(self.current_attr_name.clone(), self.current_attr_value.clone());
            self.current_attr_name.clear();
            self.current_attr_value.clear();
        }
    }
    
    fn push_attribute_name(&mut self, s: &[u8]) {
        self.update_span(s);
        self.current_attr_name.push_str(&String::from_utf8_lossy(s));
    }
    
    fn push_attribute_value(&mut self, s: &[u8]) {
        self.update_span(s);
        self.current_attr_value.push_str(&String::from_utf8_lossy(s));
    }
    
    fn set_doctype_public_identifier(&mut self, _: &[u8]) {}
    fn set_doctype_system_identifier(&mut self, _: &[u8]) {}
    fn push_doctype_public_identifier(&mut self, _: &[u8]) {}
    fn push_doctype_system_identifier(&mut self, _: &[u8]) {}
    fn current_is_appropriate_end_tag_token(&mut self) -> bool { false }
}
