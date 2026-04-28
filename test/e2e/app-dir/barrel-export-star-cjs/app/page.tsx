import { Button, LEGACY_CONST } from '../components/barrel'
import * as barrelNs from '../components/barrel'

export default function Page() {
  // Forces an `ownKeys` call on the dynamic-export proxy, which without
  // the runtime fix throws:
  //   TypeError: 'ownKeys' on proxy: trap returned extra keys but proxy
  //              target is non-extensible
  const keys = Object.keys(barrelNs).sort()
  return (
    <div>
      <Button />
      <p id="legacy">{LEGACY_CONST as string}</p>
      <p id="keys">{keys.join(',')}</p>
    </div>
  )
}
