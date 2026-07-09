import type { Router } from 'vue-router'

export function goBack(router: Router, fallback: string): void {
  if (window.history.state?.back) {
    router.back()
  } else {
    router.push(fallback)
  }
}
