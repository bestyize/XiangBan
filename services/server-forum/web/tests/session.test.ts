import test from 'node:test'
import assert from 'node:assert/strict'
import { SessionScope } from '../src/lib/sessionScope.ts'
function deferred<T>() {
  let resolve!: (value:T) => void, reject!: (reason:unknown) => void
  const promise = new Promise<T>((yes,no) => { resolve = yes; reject = no })
  return {promise,resolve,reject}
}

test('concurrent token requests share one refresh in the same identity',async () => {
  const scope = new SessionScope(), result = deferred<string>()
  let calls = 0
  const work = async (active:()=>boolean) => { calls++; const token = await result.promise; return active() ? token : null }
  const first = scope.refresh(work), second = scope.refresh(work)
  assert.equal(first,second)
  await Promise.resolve()
  assert.equal(calls,1)
  result.resolve('new-access-token')
  assert.deepEqual(await Promise.all([first,second]),['new-access-token','new-access-token'])
})

test('old refresh success cannot resurrect logout or overwrite a new account',async () => {
  const scope = new SessionScope(), old = deferred<string>(), next = deferred<string>()
  let stored:string|null = 'account-a'
  const first = scope.refresh(async active => { const result = await old.promise; if (!active()) return null; stored = result; return result })
  await Promise.resolve()
  scope.advance(); stored = null // logout
  scope.advance(); stored = 'account-b' // subsequent sign-in
  const second = scope.refresh(async active => { const result = await next.promise; if (!active()) return null; stored = result; return result })
  await Promise.resolve()
  old.resolve('stale-a')
  assert.equal(await first,null)
  assert.equal(stored,'account-b')
  // The old promise's finally must not remove the new account's in-flight refresh.
  const same = scope.refresh(async()=> 'duplicate')
  assert.equal(same,second)
  next.resolve('fresh-b')
  assert.equal(await second,'fresh-b')
  assert.equal(stored,'fresh-b')
})

test('old refresh error cannot clear a new identity',async () => {
  const scope = new SessionScope(), old = deferred<string>()
  let stored:string|null = 'account-a'
  const pending = scope.refresh(async active => {
    try { return await old.promise } catch { if (active()) stored = null; return null }
  })
  await Promise.resolve()
  scope.advance(); stored = 'account-b'
  old.reject(new Error('refresh rejected'))
  await pending
  assert.equal(stored,'account-b')
})

test('profile results require both the current account and latest request',async () => {
  const scope = new SessionScope(), old = deferred<string>()
  let profile = ''
  const first = scope.profileRequest()
  const pending = old.promise.then(result => { if (first()) profile = result })
  const second = scope.profileRequest()
  assert.equal(first(),false)
  if (second()) profile = 'newer-a-profile'
  old.resolve('stale-a-profile'); await pending
  assert.equal(profile,'newer-a-profile')
  const beforeSwitch = scope.profileRequest()
  scope.advance()
  assert.equal(beforeSwitch(),false)
  const current = scope.profileRequest()
  assert.equal(current(),true)
})

test('logout before queued refresh starts never calls the old endpoint',async () => {
  const scope = new SessionScope()
  let called = false
  const pending = scope.refresh(async()=> { called = true; return 'old' })
  scope.advance()
  assert.equal(await pending,null)
  assert.equal(called,false)
})

test('newer login attempts and logout invalidate older sign-in results without disrupting refresh',async () => {
  const scope = new SessionScope(), token = deferred<string>()
  const refresh = scope.refresh(async active => { const value = await token.promise; return active() ? value : null })
  await Promise.resolve()
  const firstLogin = scope.loginAttempt()
  const newerLogin = scope.loginAttempt()
  assert.equal(firstLogin(),false)
  assert.equal(newerLogin(),true)
  token.resolve('valid-current-account-refresh')
  assert.equal(await refresh,'valid-current-account-refresh')
  scope.advance() // logout invalidates even a still-pending successful password response
  assert.equal(newerLogin(),false)
})
