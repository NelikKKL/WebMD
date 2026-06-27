# WebMD

Быстрый Markdown-рендерер для браузера. Написан на **Rust**, скомпилирован в **WebAssembly** — парсинг и рендеринг происходят на скорости нативного кода прямо в браузере, без серверов и зависимостей.

Поддерживает **CommonMark** и **GFM** (GitHub-Flavored Markdown) в полном объёме, плюс собственный расширенный синтаксис для стилизации текста.

---

## Установка

Скачай два файла из релиза и положи рядом с HTML:

```
webmd.js
webmd_bg.wasm
```

`webmd_min.js` — минифицированная версия, функционально идентична.

---

## Подключение в HTML

```html
<div id="output"></div>

<script type="module">
  import init, { render } from './webmd.js';

  await init(); // загружает webmd_bg.wasm автоматически

  const html = render('# Привет\n\nЭто **WebMD**.');
  document.getElementById('output').innerHTML = html;
</script>
```

> Страница должна открываться через HTTP-сервер (`python3 -m http.server`, `npx serve` и т.д.), а не напрямую через `file://`, — браузеры блокируют загрузку `.wasm` по `file://`.

### Класс `WebMD` (переиспользуемый экземпляр)

Если рендеришь много документов, удобнее создать один объект:

```html
<script type="module">
  import init, { WebMD } from './webmd.js';

  await init();

  const md = new WebMD();

  console.log(md.render('**жирный текст**'));
  console.log(md.render('~~зачёркнутый~~'));
</script>
```

### Получение AST

```html
<script type="module">
  import init, { WebMD } from './webmd.js';

  await init();

  const md = new WebMD();
  const ast = JSON.parse(md.parse_to_json('# Заголовок\n\nАбзац.'));
  console.log(ast);
  // [
  //   { type: "heading", level: 1, children: [...] },
  //   { type: "paragraph", children: [...] }
  // ]
</script>
```

---

## Расширенный синтаксис

WebMD добавляет два инлайн-расширения поверх стандартного Markdown.

### Градиентный текст — `#grad`

```
#grad[<цвет от>, <цвет до>](текст)
```

Оборачивает текст в `<span>` с CSS `linear-gradient`. Поддерживается любой валидный CSS-цвет.

```markdown
#grad[red, blue](Радужный заголовок)

#grad[#ff6b6b, #feca57](Закат)

#grad[oklch(60% 0.25 280), oklch(60% 0.25 180)](Фиолетово-бирюзовый)

Текст абзаца с #grad[tomato, dodgerblue](**жирным градиентом**) внутри.
```

Генерируемый HTML:

```html
<span style="background: linear-gradient(to right, red, blue);
             -webkit-background-clip: text;
             -webkit-text-fill-color: transparent;
             background-clip: text;">Радужный заголовок</span>
```

---

### Шрифт — `#font`

```
#font[<семейство>](текст)
```

Устанавливает `font-family` через инлайновый `<span>`. Имена с пробелами писать без кавычек — рендерер добавит их сам.

```markdown
#font[Georgia](Текст шрифтом Georgia)

#font[Comic Sans MS](Весёлый текст)

#font[JetBrains Mono](Моноширинный без блока кода)
```

Генерируемый HTML:

```html
<span style="font-family: 'Comic Sans MS'">Весёлый текст</span>
```

---

### Комбинирование

`#grad` и `#font` можно вкладывать друг в друга:

```markdown
#font[Georgia](#grad[gold, crimson](Стильный и цветной))
```

---

## Поддерживаемый стандартный синтаксис

| Возможность | Синтаксис |
|---|---|
| Заголовки ATX | `# H1` … `###### H6` |
| Заголовки Setext | подчёркивание `===` / `---` |
| Жирный | `**текст**` или `__текст__` |
| Курсив | `*текст*` или `_текст_` |
| Жирный курсив | `***текст***` |
| Зачёркивание (GFM) | `~~текст~~` |
| Инлайн-код | `` `код` `` |
| Блок кода (с языком) | ` ```js … ``` ` |
| Блок кода (отступ) | 4 пробела |
| Ссылка | `[текст](url)` |
| Изображение | `![alt](src)` |
| Список маркированный | `- / * / +` |
| Список нумерованный | `1. 2. …` |
| Чеклист (GFM) | `- [ ] / - [x]` |
| Цитата | `> текст` |
| Таблица (GFM) | `\| A \| B \|` |
| Горизонтальная линия | `---` / `***` / `___` |
| Сырой HTML | passthrough |
| Перенос строки | два пробела или `\` в конце строки |
| Экранирование | `\*` `\_` и др. |

---

## API

### `render(markdown: string): string`

Однократный рендер. Принимает строку Markdown, возвращает HTML-строку.

```js
import init, { render } from './webmd.js';
await init();

const html = render('# Hello');
```

### `new WebMD()`

Создаёт переиспользуемый экземпляр рендерера.

```js
const md = new WebMD();
md.render(markdown);          // → HTML string
md.parse_to_json(markdown);   // → JSON string (AST)
```

### `initSync(module: BufferSource | WebAssembly.Module)`

Синхронная инициализация. Полезно, если WASM уже загружен заранее — например, встроен в страницу как base64.

```js
import { initSync, render } from './webmd.js';

const bytes = /* Uint8Array с содержимым webmd_bg.wasm */;
initSync(bytes);

const html = render('**готово**');
```

---

## Лицензия

См. `LICENSE`.
