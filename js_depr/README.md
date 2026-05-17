# contexting

A small TypeScript library for building prompts from templates, ordered injectors, and context.

## Install

```bash
npm install contexting
```

## Core concept

Define a template with `{{slot}}` placeholders. Create injectors that decide what fills each slot based on runtime context. Call `assemble` to produce the final string.

```ts
import { assemble, createInjector } from 'contexting';

const basePrompt = '{{llm_spec}}, {{name}}, and {{language}}';

const llmSpec = createInjector('llm_spec', (ctx) => {
  switch (ctx.llm) {
    case 'sonnet': return 'you are a smart llm model';
    case 'gpt5':   return 'you need to be very precise';
    default:       return 'be precise';
  }
});

assemble(basePrompt, [llmSpec], { llm: 'sonnet', language: 'klingon', name: 'KDS' });
// → 'you are a smart llm model, KDS, and klingon'
```

## Injection modes

Injectors have two behaviours depending on whether the slot appears in the template:

| Slot in template? | Result |
|---|---|
| Yes | Injector output replaces `{{slot}}` inline |
| No | Injector output is appended after a blank line |

```ts
const basePrompt = 'You are a {{role}} assistant for {{lang}}.';

const roleInjector = createInjector('role', (ctx) =>
  ctx.senior ? 'senior software engineer' : 'helpful coding'
);

// {{constraints}} is not in the template — output will be appended
const constraintsInjector = createInjector('constraints', (ctx) =>
  ctx.safe ? 'Never suggest running untrusted code. Always explain risks.' : ''
);

const langInjector = createInjector('lang', (ctx) => {
  switch (ctx.lang) {
    case 'js':
    case 'javascript': return 'JavaScript, Node and TypeScript';
    default:           return 'Python and Java';
  }
});

assemble(basePrompt, [roleInjector, constraintsInjector, langInjector], {
  senior: true,
  safe: true,
  lang: 'javascript',
});
// → 'You are a senior software engineer assistant for JavaScript, Node and TypeScript.'
// → ''
// → 'Never suggest running untrusted code. Always explain risks.'
```

After all injectors run, any remaining `{{slot}}` placeholders are filled directly from context if the value is a string or number.

## API

### `assemble(basePrompt, rules, context)`

| Param | Type | Description |
|---|---|---|
| `basePrompt` | `string` | Template with `{{slot}}` placeholders |
| `rules` | `Injector[]` | Ordered injectors to apply |
| `context` | `Record<string, unknown>` | Data available to every injector and base fill |

Returns the assembled `string`.

### `createInjector(slot, fn)`

| Param | Type | Description |
|---|---|---|
| `slot` | `string` | Placeholder name without braces, e.g. `'role'` for `{{role}}` |
| `fn` | `(ctx: Context) => string` | Produces the injection text from context |

Returns an `Injector`. Throws `TypeError` if `slot` is empty.

## Types

```ts
type BasePrompt  = string;
type Context     = Record<string, unknown>;
type InjectorFn  = (context: Context) => string;
type Injector    = { slot: string; fn: InjectorFn };
type Rules       = Injector[];
```

## Build

```bash
npm run build   # emits dist/ with JS + .d.ts declarations
npm run dev     # watch mode
```

---

This codebase was built using the alpha version of the [Sage ecosystem](https://actuallygood.club).
