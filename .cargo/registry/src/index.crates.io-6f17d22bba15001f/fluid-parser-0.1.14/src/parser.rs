use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    pub i: usize,
    pub tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let mut t = lexer.next();
        let mut tokens = vec![t];
        while t.typ != TokenType::Eof {
            t = lexer.next();
            tokens.push(t);
        }
        Self {
            lexer,
            i: 0,
            tokens,
        }
    }
    pub fn parse(&mut self) -> Ast {
        let mut a = Ast::default();
        while self.i < self.tokens.len() {
            let curr = self.tokens[self.i];
            match curr.typ {
                TokenType::Eof => break,
                TokenType::Word => match curr.word {
                    "i18n_type" => {
                        a.i18n_type = Some(true);
                        self.i += 2;
                    }
                    "class" => {
                        let c = self.consume_class();
                        a.classes.push(c);
                    }
                    "Function" => {
                        let f = self.consume_func();
                        a.functions.push(f);
                    }
                    "comment" => {
                        let c = self.consume_comment();
                        a.comments.push(c);
                    }
                    "decl" => {
                        let d = self.consume_decl();
                        a.decls.push(d);
                    }
                    "widget_class" => {
                        let w = self.consume_widget();
                        a.widget_classes.push(w);
                    }
                    _ => (),
                },
                _ => (),
            }
            self.i += 1;
        }
        a
    }
    fn consume_func(&mut self) -> Function {
        let mut f = Function::default();
        self.i += 1;
        f.name = self.consume_braced_string();
        self.i += 1; // opening parens of props
        while self.tokens[self.i].typ != TokenType::Eof {
            self.i += 1;
            if self.tokens[self.i].typ == TokenType::CloseBrace {
                break;
            }
            match self.tokens[self.i].word {
                "open" => f.props.open = Some(true),
                "C" => f.props.c = Some(true),
                "protected" => f.props.visibility = Some(Visibility::PROTECTED),
                "private" => f.props.visibility = Some(Visibility::PRIVATE),
                "comment" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        f.props.comment = Some(self.consume_braced_string());
                    } else {
                        f.props.comment = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "return_type" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        f.props.return_type = Some(self.consume_braced_string());
                    } else {
                        f.props.return_type = Some(self.tokens[self.i].word.to_string());
                    }
                }
                _ => (),
            }
        }
        self.i += 1; // close props parens
        if self.tokens[self.i].typ == TokenType::OpenBrace {
            while self.tokens[self.i].typ != TokenType::CloseBrace {
                self.i += 1;
                if self.tokens[self.i].word == "code" {
                    self.i += 1;
                    f.code = Some(self.consume_code());
                    self.i += 1;
                }
                if self.tokens[self.i].word.starts_with("Fl_")
                    || self.tokens[self.i].word == "MenuItem"
                    || self.tokens[self.i].word == "Submenu"
                {
                    let w = self.consume_widget();
                    f.widgets.push(w);
                    self.i += 1;
                }
            }
        }
        f
    }
    fn consume_widget(&mut self) -> Widget {
        let mut w = Widget {
            typ: self.tokens[self.i].word.to_string(),
            ..Default::default()
        };
        self.i += 1;
        if self.tokens[self.i].typ == TokenType::OpenBrace {
            self.i += 1;
        }
        if !self.tokens[self.i].word.is_empty() {
            w.name = self.tokens[self.i].word.to_string();
        } else {
            w.name = String::new();
        }
        while self.tokens[self.i].typ != TokenType::Eof {
            self.i += 1;
            if self.tokens[self.i].typ == TokenType::CloseBrace {
                break;
            }
            match self.tokens[self.i].word {
                "open" => w.props.open = Some(true),
                "hide" => w.props.hide = Some(true),
                "deactivate" => w.props.deactivate = Some(true),
                "divider" => w.props.divider = Some(true),
                "resizable" => w.props.resizable = Some(true),
                "visible" => w.props.visible = Some(true),
                "hotspot" => w.props.hotspot = Some(true),
                "modal" => w.props.modal = Some(true),
                "non_modal" => w.props.non_modal = Some(true),
                "noborder" => w.props.noborder = Some(true),
                "xywh" => {
                    self.i += 1;
                    w.props.xywh = self.consume_braced_string();
                }
                "size_range" => {
                    self.i += 1;
                    w.props.size_range = Some(self.consume_braced_string());
                }
                "color" => {
                    self.i += 1;
                    w.props.color = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `align` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "selection_color" => {
                    self.i += 1;
                    w.props.selection_color = Some(
                        self.tokens[self.i]
                            .word
                            .to_string()
                            .parse()
                            .unwrap_or_else(|_| panic!("Failed to parse numeric value for `selection_color` property at token:\"{:?}\"!",
                            self.tokens[self.i])),
                    );
                }
                "labelcolor" => {
                    self.i += 1;
                    w.props.labelcolor = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `labelcolor` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "textcolor" => {
                    self.i += 1;
                    w.props.textcolor = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `textcolor` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "type" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.typ = Some(self.consume_braced_string());
                    } else {
                        w.props.typ = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "labeltype" => {
                    self.i += 1;
                    w.props.labeltype = Some(self.tokens[self.i].word.to_string());
                }
                "labelfont" => {
                    self.i += 1;
                    w.props.labelfont = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `labeltype` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "textfont" => {
                    self.i += 1;
                    w.props.textfont = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `textfont` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "labelsize" => {
                    self.i += 1;
                    w.props.labelsize = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `labelsize` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "textsize" => {
                    self.i += 1;
                    w.props.textsize = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `textsize` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "box" => {
                    self.i += 1;
                    w.props.r#box = Some(self.tokens[self.i].word.to_string());
                }
                "down_box" => {
                    self.i += 1;
                    w.props.down_box = Some(self.tokens[self.i].word.to_string());
                }
                "align" => {
                    self.i += 1;
                    w.props.align = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `align` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "when" => {
                    self.i += 1;
                    w.props.when = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `when` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "shortcut" => {
                    self.i += 1;
                    w.props.shortcut = Some(self.tokens[self.i].word.to_string());
                }
                "gap" => {
                    self.i += 1;
                    w.props.gap = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `gap` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "minimum" => {
                    self.i += 1;
                    w.props.minimum = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `minimum` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "maximum" => {
                    self.i += 1;
                    w.props.maximum = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `maximum` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "step" => {
                    self.i += 1;
                    w.props.step = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `step` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "slider_size" => {
                    self.i += 1;
                    w.props.slider_size = Some(
                        self.tokens[self.i]
                            .word
                            .to_string()
                            .parse()
                            .unwrap_or_else(|_| panic!("Failed to parse numeric value for `slider_size` property at token:\"{:?}\"!",
                            self.tokens[self.i])),
                    );
                }
                "size" => {
                    self.i += 1;
                    w.props.size = Some(self.tokens[self.i].word.to_string().parse().unwrap_or_else(|_| panic!("Failed to parse numeric value for `size` property at token:\"{:?}\"!",
                            self.tokens[self.i])));
                }
                "label" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.label = Some(self.consume_braced_string());
                    } else {
                        w.props.label = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "xclass" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.xclass = Some(self.consume_braced_string());
                    } else {
                        w.props.xclass = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "class" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.class = Some(self.consume_braced_string());
                    } else {
                        w.props.class = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "tooltip" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.tooltip = Some(self.consume_braced_string());
                    } else {
                        w.props.tooltip = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "image" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.image = Some(self.consume_braced_string());
                    } else {
                        w.props.image = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "deimage" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.deimage = Some(self.consume_braced_string());
                    } else {
                        w.props.deimage = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "value" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.value = Some(self.consume_braced_string());
                    } else {
                        w.props.value = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "set_size_tuples" => {
                    self.i += 1;
                    w.props.size_tuple = Some(self.consume_braced_string());
                }
                "margins" => {
                    self.i += 1;
                    w.props.margins = Some(self.consume_braced_string());
                }
                "fixed_size_tuples" => {
                    self.i += 1;
                    w.props.size_tuple = Some(self.consume_braced_string());
                }
                "code0" => {
                    self.i += 1;
                    w.props.code0 = Some(self.consume_braced_string());
                }
                "code1" => {
                    self.i += 1;
                    w.props.code1 = Some(self.consume_braced_string());
                }
                "code2" => {
                    self.i += 1;
                    w.props.code2 = Some(self.consume_braced_string());
                }
                "code3" => {
                    self.i += 1;
                    w.props.code3 = Some(self.consume_braced_string());
                }
                "extra_code" => {
                    self.i += 1;
                    w.props.extra_code = Some(self.consume_braced_string());
                }
                "callback" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.callback = Some(self.consume_braced_string());
                    } else {
                        w.props.callback = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "user_data" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.user_data = Some(self.consume_braced_string());
                    } else {
                        w.props.user_data = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "user_data_type" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.user_data_type = Some(self.consume_braced_string());
                    } else {
                        w.props.user_data_type = Some(self.tokens[self.i].word.to_string());
                    }
                }
                "comment" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        w.props.comment = Some(self.consume_braced_string());
                    } else {
                        w.props.comment = Some(self.tokens[self.i].word.to_string());
                    }
                }
                _ => (),
            }
        }
        if self.tokens[self.i + 1].typ == TokenType::OpenBrace {
            self.i += 1;
            while self.tokens[self.i].typ != TokenType::CloseBrace {
                self.i += 1;
                while self.tokens[self.i].word.starts_with("Fl_")
                    || self.tokens[self.i].word == "MenuItem"
                    || self.tokens[self.i].word == "Submenu"
                {
                    let c = self.consume_widget();
                    w.children.push(c);
                    self.i += 1;
                }
            }
        }
        w
    }
    fn consume_class(&mut self) -> Class {
        let mut c = Class::default();
        self.i += 1;
        if self.tokens[self.i].typ == TokenType::OpenBrace {
            self.i += 2;
        }
        c.name = self.tokens[self.i].word.to_string();
        self.i += 1;
        // handle props
        while self.tokens[self.i].typ != TokenType::CloseBrace {
            self.i += 1;
            match self.tokens[self.i].word {
                "open" => c.props.open = Some(true),
                "protected" => c.props.visibility = Some(Visibility::PROTECTED),
                "private" => c.props.visibility = Some(Visibility::PRIVATE),
                "comment" => {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        c.props.comment = Some(self.consume_braced_string());
                    } else {
                        c.props.comment = Some(self.tokens[self.i].word.to_string());
                    }
                }
                _ => (),
            }
        }
        self.i += 1;
        if self.tokens[self.i].typ == TokenType::OpenBrace {
            while self.tokens[self.i].typ != TokenType::CloseBrace {
                self.i += 1;
                while self.tokens[self.i].word == "Function" {
                    let f = self.consume_func();
                    c.functions.push(f);
                }
                if self.tokens[self.i].word == "comment" {
                    self.i += 1;
                    if self.tokens[self.i].typ == TokenType::OpenBrace {
                        c.props.comment = Some(self.consume_braced_string());
                    } else {
                        c.props.comment = Some(self.tokens[self.i].word.to_string());
                    }
                }
            }
        }
        c
    }
    fn consume_comment(&mut self) -> Comment {
        let mut c = Comment::default();
        self.i += 1;
        c.comment = self.consume_braced_string();
        while self.tokens[self.i].typ != TokenType::Eof {
            self.i += 1;
            if self.tokens[self.i].typ == TokenType::CloseBrace {
                break;
            }
            match self.tokens[self.i].word {
                "in_source" => c.props.in_source = Some(true),
                "in_header" => c.props.in_header = Some(true),
                _ => (),
            }
        }
        c
    }
    fn consume_decl(&mut self) -> Decl {
        let mut d = Decl::default();
        self.i += 1;
        d.decl = self.consume_braced_string();
        while self.tokens[self.i].typ != TokenType::Eof {
            self.i += 1;
            if self.tokens[self.i].typ == TokenType::CloseBrace {
                break;
            }
            match self.tokens[self.i].word {
                "private" => d.props.visibility = Visibility::PRIVATE,
                "public" => d.props.visibility = Visibility::PUBLIC,
                "global" => d.props.global = Some(true),
                "local" => d.props.local = Some(true),
                _ => (),
            }
        }
        d
    }
    fn consume_code(&mut self) -> String {
        let s = self.consume_braced_string();
        // skip last 2 braces
        self.i += 1;
        self.i += 1;
        s
    }

    fn consume_braced_string(&mut self) -> String {
        self.i += 1;
        let mut t = self.tokens[self.i];
        let start = t.start;
        let mut openbrace = 1;
        while t.typ != TokenType::Eof {
            self.i += 1;
            t = self.tokens[self.i];
            if t.typ == TokenType::OpenBrace {
                openbrace += 1;
            }
            if t.typ == TokenType::CloseBrace {
                openbrace -= 1;
            }
            if openbrace == 0 {
                break;
            }
        }
        let end = self.tokens[self.i].end - 1;
        self.lexer.s[start..end].to_string()
    }
}
