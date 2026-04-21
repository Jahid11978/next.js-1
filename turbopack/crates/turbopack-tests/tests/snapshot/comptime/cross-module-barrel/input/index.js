import { SOME_VALUE, BARREL_VALUE, foo } from './library'

// TODO ideally this would be inlined (reexport resolving with constants)
if (SOME_VALUE === 'x') {
  console.log('x')
} else {
  require('./dead-code')
}
console.log(SOME_VALUE)

console.log(foo())

if (BARREL_VALUE === 'barrel') {
  console.log('x')
} else {
  require('./dead-code')
}
console.log(BARREL_VALUE)
