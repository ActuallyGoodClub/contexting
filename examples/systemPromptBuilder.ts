import { assemble, createInjector } from 'contexting';

const basePrompt = 'You are a {{role}} assistant for the language {{lang}}.';

// Inline injection: {{role}} exists in template, so the output replaces it directly.
const roleInjector = createInjector('role', (ctx) =>
  ctx.senior ? 'senior software engineer' : 'helpful coding'
);

// Appended injection: {{constraints}} is absent, so the output is appended after a blank line.
const constraintsInjector = createInjector('constraints', (ctx) =>
  ctx.safe ? 'Never suggest running untrusted code. Always explain risks.' : ''
);

const injectLangCtx = createInjector('lang', (ctx) => {
  switch(ctx.lang) {
    case "javascript": {
      return "javascript, node and typescript"
    }
    default: {
      return "python and java";
    }
  }
});

const prompt = assemble(basePrompt, [roleInjector, constraintsInjector, injectLangCtx], {
  senior: true,
  safe: true,
  lang: "javascript",
});
console.log({prompt});

const promptWithDiffLang = assemble(basePrompt, [roleInjector, constraintsInjector, injectLangCtx], {
  senior: true,
  safe: true,
  lang: "python",
});
console.log({promptWithDiffLang});


const promptWithoutLangInjection = assemble(basePrompt, [roleInjector, constraintsInjector], {
  senior: true,
  safe: true,
  lang: "python",
});
console.log({promptWithoutLangInjection});