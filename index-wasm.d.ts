// TypeScript definitions for nools-rust WASM

export class Fact {
  constructor(data: string);
  readonly id: number;
  data: string;
}

export class Flow {
  constructor(name: string);
  readonly name: string;
  readonly ruleCount: number;
  
  add_rule(name: string, priority: number): RuleBuilder;
  session(): Session;
}

export class RuleBuilder {
  when(condition: string): RuleBuilder;
}

export class Session {
  readonly factCount: number;
  readonly halted: boolean;
  
  assert(fact: Fact): void;
  retract(factId: number): boolean;
  get_facts(): any;
  match_rules(): number;
  halt(): void;
  dispose(): void;
}

export function flow(name: string): Flow;
export function version(): string;
export function init(): void;
