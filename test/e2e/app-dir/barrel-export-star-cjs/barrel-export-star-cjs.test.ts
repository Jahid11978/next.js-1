import { nextTestSetup } from 'e2e-utils'

// Regression test for a Turbopack runtime bug: when a barrel module re-exports
// from a CommonJS module via `export *`, the runtime wraps `module.exports` in
// a Proxy whose `ownKeys` trap synthesizes the dynamic keys. `esm()` then calls
// `Object.seal` on that namespace to enforce ESM immutability, which violates
// the proxy invariant ("trap returned extra keys but proxy target is non-
// extensible") either at module evaluation or at the first consumer-side
// `Object.keys` / `for..in` / spread.
describe('app-dir - barrel export * from cjs', () => {
  const { next } = nextTestSetup({
    files: __dirname,
  })

  it('renders a barrel that re-exports from a cjs module without a proxy invariant error', async () => {
    const $ = await next.render$('/')
    expect($('#legacy').text()).toBe('i am dynamic')
    expect($('#keys').text().split(',')).toEqual(
      expect.arrayContaining(['Button', 'LEGACY_CONST', 'AnotherThing'])
    )
  })
})
