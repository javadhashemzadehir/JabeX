import type { InkDocument } from "@jabex/types";
import { cloneInkDocument } from "./serialize";

export class InkHistory {
  private undoStack: InkDocument[] = [];
  private redoStack: InkDocument[] = [];

  constructor(
    private current: InkDocument,
    private readonly limit = 50,
  ) {}

  get document(): InkDocument {
    return this.current;
  }

  canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  commit(next: InkDocument) {
    this.undoStack.push(cloneInkDocument(this.current));
    if (this.undoStack.length > this.limit) this.undoStack.shift();
    this.current = next;
    this.redoStack = [];
  }

  replace(next: InkDocument) {
    this.current = next;
  }

  undo(): InkDocument | null {
    const prev = this.undoStack.pop();
    if (!prev) return null;
    this.redoStack.push(cloneInkDocument(this.current));
    this.current = prev;
    return this.current;
  }

  redo(): InkDocument | null {
    const next = this.redoStack.pop();
    if (!next) return null;
    this.undoStack.push(cloneInkDocument(this.current));
    this.current = next;
    return this.current;
  }
}
