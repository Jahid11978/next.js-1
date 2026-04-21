import { lower } from './other' with { turbopackConstants: 'true' }

if (lower === 'lowercase') {
  console.log('x')
} else {
  require('./dead-code')
}
console.log(lower)
