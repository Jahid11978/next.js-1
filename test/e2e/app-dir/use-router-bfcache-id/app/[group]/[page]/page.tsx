'use client'

import { useRouter, usePathname, useSearchParams } from 'next/navigation'
import { refreshAction } from '../../actions'

export default function LeafPage() {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const { bfcacheId } = router
  const search = searchParams.toString()
  return (
    <section>
      <h1 data-testid="pathname">{pathname}</h1>
      <span data-testid="search" data-value={search}>
        {search}
      </span>
      <form key={bfcacheId}>
        <input data-testid="leaf-input" defaultValue="" />
      </form>
      <button
        data-testid="push-search"
        onClick={() => router.push(`${pathname}?q=2`)}
      >
        push search
      </button>
      <button
        data-testid="push-hash"
        onClick={() => router.push(`${pathname}#section`)}
      >
        push hash
      </button>
      <button data-testid="refresh" onClick={() => router.refresh()}>
        refresh
      </button>
      <form action={refreshAction}>
        <button data-testid="server-action-refresh" type="submit">
          server action refresh
        </button>
      </form>
    </section>
  )
}
