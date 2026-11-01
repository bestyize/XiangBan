/** Async work belongs to an identity generation; advancing invalidates every old result. */
export class SessionScope {
  private generation = 0
  private profileSequence = 0
  private loginSequence = 0
  private refreshing: { generation:number; promise:Promise<string|null> } | null = null

  advance(): number {
    ++this.generation
    ++this.profileSequence
    ++this.loginSequence
    this.refreshing = null
    return this.generation
  }
  capture(): number { return this.generation }
  current(generation:number): boolean { return generation === this.generation }
  loginAttempt(): () => boolean {
    const generation = this.generation, sequence = ++this.loginSequence
    return () => this.current(generation) && sequence === this.loginSequence
  }
  profileRequest(): () => boolean {
    const generation = this.generation, sequence = ++this.profileSequence
    return () => this.current(generation) && sequence === this.profileSequence
  }
  refresh(work:(current:() => boolean) => Promise<string|null>): Promise<string|null> {
    if (this.refreshing?.generation === this.generation) return this.refreshing.promise
    const generation = this.generation
    const task = {generation, promise:Promise.resolve(null) as Promise<string|null>}
    // Scheduling work in a microtask installs the shared promise before work can complete.
    task.promise = Promise.resolve().then(() => this.current(generation) ? work(() => this.current(generation)) : null).finally(() => {
      if (this.refreshing === task) this.refreshing = null
    })
    this.refreshing = task
    return task.promise
  }
}
