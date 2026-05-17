/** A prompt template string containing `{{slot}}` placeholders. */
export type BasePrompt = string;

/** Arbitrary key-value data passed to injectors and used to fill remaining slots. */
export type Context = Record<string, unknown>;

/** A function that receives `context` and returns the string to inject. */
export type InjectorFn = (context: Context) => string;

/** An injector binding a slot name to its producer function. */
export type Injector = { slot: string; fn: InjectorFn };

/** An ordered list of injectors applied by `assemble`. */
export type Rules = Injector[];
