#[derive(Default, Debug)]
pub struct WidgetProps {
    pub open: Option<bool>,
    pub xywh: String,
    pub label: Option<String>,
    pub typ: Option<String>,
    pub hide: Option<bool>,
    pub deactivate: Option<bool>,
    pub visible: Option<bool>,
    pub modal: Option<bool>,
    pub non_modal: Option<bool>,
    pub noborder: Option<bool>,
    pub xclass: Option<String>,
    pub size_range: Option<String>,
    pub resizable: Option<bool>,
    pub hotspot: Option<bool>,
    pub divider: Option<bool>,
    pub selected: Option<bool>,
    pub color: Option<u32>,
    pub selection_color: Option<u32>,
    pub tooltip: Option<String>,
    pub image: Option<String>,
    pub deimage: Option<String>,
    pub r#box: Option<String>,
    pub down_box: Option<String>,
    pub value: Option<String>,
    pub labeltype: Option<String>,
    pub labelfont: Option<i32>,
    pub labelsize: Option<i32>,
    pub labelcolor: Option<u32>,
    pub align: Option<i32>,
    pub when: Option<i32>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub step: Option<f64>,
    pub slider_size: Option<f64>,
    pub size: Option<f64>,
    pub textfont: Option<i32>,
    pub textsize: Option<i32>,
    pub textcolor: Option<u32>,
    pub class: Option<String>,
    pub shortcut: Option<String>,
    pub code0: Option<String>,
    pub code1: Option<String>,
    pub code2: Option<String>,
    pub code3: Option<String>,
    pub extra_code: Option<String>,
    pub size_tuple: Option<String>,
    pub margins: Option<String>,
    pub gap: Option<i32>,
    pub user_data: Option<String>,
    pub user_data_type: Option<String>,
    pub callback: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Default)]
pub struct Widget {
    pub typ: String,
    pub name: String,
    pub props: WidgetProps,
    pub children: Vec<Widget>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Visibility {
    PUBLIC,
    PRIVATE,
    PROTECTED,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::PUBLIC
    }
}

#[derive(Debug, Default)]
pub struct FunctionProps {
    pub visibility: Option<Visibility>,
    pub open: Option<bool>,
    pub c: Option<bool>,
    pub return_type: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Default)]
pub struct Function {
    pub name: String,
    pub props: FunctionProps,
    pub code: Option<String>,
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Default)]
pub struct ClassProps {
    pub visibility: Option<Visibility>,
    pub open: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Debug, Default)]
pub struct Class {
    pub name: String,
    pub props: ClassProps,
    pub functions: Vec<Function>,
}

#[derive(Debug, Default)]
pub struct DeclProps {
    pub visibility: Visibility,
    pub global: Option<bool>,
    pub local: Option<bool>,
}

#[derive(Debug, Default)]
pub struct Decl {
    pub decl: String,
    pub props: DeclProps,
}

#[derive(Debug, Default)]
pub struct CommentProps {
    pub in_source: Option<bool>,
    pub in_header: Option<bool>,
}

#[derive(Debug, Default)]
pub struct Comment {
    pub comment: String,
    pub props: CommentProps,
}

#[derive(Debug, Default)]
pub struct Ast {
    pub i18n_type: Option<bool>,
    pub classes: Vec<Class>,
    pub widget_classes: Vec<Widget>,
    pub functions: Vec<Function>,
    pub comments: Vec<Comment>,
    pub decls: Vec<Decl>,
}
