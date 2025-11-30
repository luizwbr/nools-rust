// TypeScript definitions for nools-rust

export interface Message {
  text: string;
}

export interface FlowOptions {
  name: string;
}

export interface RuleOptions {
  priority?: number;
  agendaGroup?: string;
  autoFocus?: boolean;
}

export class Flow {
  constructor(name: string);
  
  /**
   * Add a rule to the flow
   */
  addRule(name: string, options: RuleOptions, callback: (facts: any) => void): void;
  
  /**
   * Create a new session
   */
  session(): Session;
}

export class Session {
  /**
   * Assert a fact into working memory
   */
  assert(fact: any): Promise<string>;
  
  /**
   * Retract a fact from working memory
   */
  retract(factId: string): Promise<void>;
  
  /**
   * Modify a fact in working memory
   */
  modify(factId: string): Promise<void>;
  
  /**
   * Set focus to an agenda group
   */
  focus(group: string): Session;
  
  /**
   * Match and fire rules once
   */
  matchRules(): Promise<number>;
  
  /**
   * Match and fire rules until halt
   */
  matchUntilHalt(): Promise<number>;
  
  /**
   * Halt rule execution
   */
  halt(): void;
  
  /**
   * Get number of facts in working memory
   */
  factCount(): number;
  
  /**
   * Dispose of the session
   */
  dispose(): void;
}

/**
 * Create a new flow
 */
export function flow(name: string): Flow;

/**
 * Get an existing flow
 */
export function getFlow(name: string): Flow | undefined;

/**
 * Check if a flow exists
 */
export function hasFlow(name: string): boolean;

/**
 * Delete a flow
 */
export function deleteFlow(name: string): void;

/**
 * Delete all flows
 */
export function deleteFlows(): void;
