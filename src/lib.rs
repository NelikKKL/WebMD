use wasm_bindgen::prelude::*;
use pulldown_cmark::{Parser, html};
use regex::Regex;

// Инициализация (опционально, для вывода логов в консоль браузера)
#[wasm_bindgen(start)]
pub fn run() {
    // Здесь можно добавить настройку паники: console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WebMD;

#[wasm_bindgen]
impl WebMD {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebMD {
        WebMD
    }

    #[wasm_bindgen]
    pub fn render(&self, markdown_input: &str) -> String {
        // 1. Препроцессинг кастомного синтаксиса градиента
        // Пример: #grad[red, blue](Мой крутой текст)
        let re_grad = Regex::new(r"#grad\[([^,\]]+),\s*([^\]]+)\]\(([^)]+)\)").unwrap();
        let with_gradients = re_grad.replace_all(
            markdown_input,
            "<span style=\"background: linear-gradient(to right, $1, $2); -webkit-background-clip: text; -webkit-text-fill-color: transparent;\">$3</span>"
        );

        // 2. Препроцессинг кастомного синтаксиса шрифтов
        // Пример: #font[Comic Sans MS](Текст с другим шрифтом)
        let re_font = Regex::new(r"#font\[([^\]]+)\]\(([^)]+)\)").unwrap();
        let processed_md = re_font.replace_all(
            &with_gradients,
            "<span style=\"font-family: '$1';\">$2</span>"
        );

        // 3. Стандартный парсинг Markdown в HTML
        let parser = Parser::new(&processed_md);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }
}