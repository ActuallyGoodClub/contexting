import { assemble, createInjector } from 'contexting';

const basePrompt = '{{lang_style}} Help the user with their question about {{topic}}.';

const langStyle = createInjector('lang_style', (ctx) => {
  switch (ctx.lang) {
    case 'es': return 'Respond only in Spanish. Be warm and conversational.';
    case 'fr': return 'Respond only in French. Be formal and precise.';
    default:   return 'Respond in English. Be clear and concise.';
  }
});

const prompt = assemble(basePrompt, [langStyle], { lang: 'es', topic: 'cooking' });
console.log(prompt);
// Respond only in Spanish. Be warm and conversational. Help the user with their question about cooking.
