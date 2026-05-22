# contexting

Template-based prompt assembly. Fill `{{slot}}` placeholders in a base prompt using ordered injectors and a runtime context.

---

## How it works

- **Inline injection** — if a slot exists in the template, the injector's output replaces it directly
- **Appended injection** — if the slot is absent, the output is collected and appended after the filled template, separated by a blank line
- **Base fill** — after all injectors run, remaining `{{slots}}` are filled directly from context (strings and numbers only)

---

## Python

### Installation

```bash
pip install contexting
```

### API

```python
from contexting import assemble, create_injector
```

#### `create_injector(slot, fn)`

| param | type | description |
|---|---|---|
| `slot` | `str` | name matching `{{slot}}` in the template |
| `fn` | `Callable[[dict], str]` | receives the full context, returns the string to inject |

#### `assemble(base_prompt, rules, context)`

| param | type | description |
|---|---|---|
| `base_prompt` | `str` | template string with `{{slot}}` placeholders |
| `rules` | `list[Injector]` | ordered injectors applied before base fill |
| `context` | `dict` | passed to every injector fn; also fills remaining slots |

Returns the assembled `str`.

---

### Examples

#### Inline and appended injection

```python
from contexting import assemble, create_injector

base_prompt = 'You are a {{role}} assistant for the language {{lang}}.'

# Inline — {{role}} exists in template, replaced directly
role_injector = create_injector('role', lambda ctx:
    'senior software engineer' if ctx.get('senior') else 'helpful coding'
)

# Appended — {{constraints}} absent, output appended after a blank line
constraints_injector = create_injector('constraints', lambda ctx:
    'Never suggest running untrusted code. Always explain risks.' if ctx.get('safe') else ''
)

lang_injector = create_injector('lang', lambda ctx:
    'javascript, node and typescript' if ctx.get('lang') == 'javascript' else 'python and java'
)

prompt = assemble(base_prompt, [role_injector, constraints_injector, lang_injector], {
    'senior': True,
    'safe': True,
    'lang': 'javascript',
})

print(prompt)
# You are a senior software engineer assistant for the language javascript, node and typescript.
#
# Never suggest running untrusted code. Always explain risks.
```

#### Base fill from context

Remaining slots not handled by any injector are filled directly from context:

```python
prompt = assemble('Hello {{name}}, you are using {{lang}}.', [], {
    'name': 'Alice',
    'lang': 'python',
})

print(prompt)
# Hello Alice, you are using python.
```

#### Language-aware prompts

```python
base_prompt = '{{lang_style}} Help the user with their question about {{topic}}.'

lang_style = create_injector('lang_style', lambda ctx: {
    'es': 'Respond only in Spanish. Be warm and conversational.',
    'fr': 'Respond only in French. Be formal and precise.',
}.get(ctx.get('lang'), 'Respond in English. Be clear and concise.'))

prompt = assemble(base_prompt, [lang_style], {'lang': 'es', 'topic': 'cooking'})

print(prompt)
# Respond only in Spanish. Be warm and conversational. Help the user with their question about cooking.
```

---

## JavaScript / WASM

### Installation

Download the `.tgz` from the [latest release](https://github.com/borkds/contexting/releases/latest) and install it:

```bash
npm install https://github.com/borkds/contexting/releases/download/v1.0.0/contexting-js-1.0.0.tgz
```

Replace the version in the URL with the release you want.

### API

```js
import { assemble, create_injector } from 'contexting-js';
```

#### `create_injector(slot, fn)`

| param | type | description |
|---|---|---|
| `slot` | `string` | name matching `{{slot}}` in the template |
| `fn` | `(ctx: object) => string` | receives the full context, returns the string to inject |

#### `assemble(base_prompt, rules, context)`

| param | type | description |
|---|---|---|
| `base_prompt` | `string` | template string with `{{slot}}` placeholders |
| `rules` | `Injector[]` | ordered injectors applied before base fill |
| `context` | `object` | passed to every injector fn; also fills remaining slots |

Returns the assembled `string`.
